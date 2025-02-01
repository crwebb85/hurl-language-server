use dashmap::DashMap;
use env_logger::Env;
use hurl_language_server::completion::{completion, ImCompleteCompletionItem};
use hurl_language_server::utils::offset_to_position;
use hurl_parser::parser::types::{Ast, Rich};
use log::debug;
use ropey::Rope;
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub const HELP: &str = "USAGE:
    hurl-language-server [OPTIONS]

To start the language server run without any arguments.

OPTIONS:
    -v, --version                   Print version info and exit
    -h, --help                      Prints help information
";

#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: DashMap<String, Option<hurl_parser::parser::types::Ast>>,
    document_map: DashMap<String, Rope>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: Some("utf-8".to_string()),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "[".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        debug!("initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("file opened");
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: &params.text_document.text,
            version: Some(params.text_document.version),
        })
        .await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            text: &params.content_changes[0].text,
            uri: params.text_document.uri,
            version: Some(params.text_document.version),
        })
        .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        dbg!(&params.text);
        if let Some(text) = params.text {
            let item = TextDocumentItem {
                uri: params.text_document.uri,
                text: &text,
                version: None,
            };
            self.on_change(item).await;
            _ = self.client.semantic_tokens_refresh().await;
        }
        debug!("file saved!");
    }
    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        debug!("file closed!");
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "completion at {uri}:{line}:{col}",
                    uri = uri,
                    line = position.line,
                    col = position.character
                ),
            )
            .await;

        let completions = {
            let completions = completion();
            let mut ret = Vec::with_capacity(completions.len());
            for (_, item) in completions {
                match item {
                    ImCompleteCompletionItem::Keyword(var) => {
                        ret.push(CompletionItem {
                            label: var.clone(),
                            insert_text: Some(var.clone()),
                            kind: Some(CompletionItemKind::KEYWORD),
                            detail: Some(var),
                            ..Default::default()
                        });
                    }
                    ImCompleteCompletionItem::Snippet(var, snippet_text) => {
                        ret.push(CompletionItem {
                            label: var.clone(),
                            kind: Some(CompletionItemKind::SNIPPET),
                            detail: Some(var.clone()),
                            insert_text: Some(snippet_text),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                }
            }
            Some(ret)
        };

        Ok(completions.map(CompletionResponse::Array))
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("configuration changed!");
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        debug!("workspace folders changed!");
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        debug!("watched files have changed!");
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        debug!("command executed!");

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }
}

#[allow(unused)]
struct TextDocumentItem<'a> {
    uri: Url,
    text: &'a str,
    version: Option<i32>,
}

impl Backend {
    async fn on_change<'a>(&self, params: TextDocumentItem<'a>) {
        dbg!(&params.version);
        let rope = ropey::Rope::from_str(params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
        debug!("about to parse document");
        debug!("document: {}", params.text);

        let mut diagnostics: Vec<Diagnostic> = vec![];
        let (ast, errs): (Option<Ast>, Vec<Rich<'_, char>>) =
            hurl_parser::parser::parser::parse_ast(params.text);

        self.ast_map.insert(params.uri.to_string(), ast);
        for err in errs {
            let span = err.span();
            let start_position = offset_to_position(span.start, &rope);
            let end_position = offset_to_position(span.end, &rope);
            let diag = start_position
                .and_then(|start| end_position.map(|end| (start, end)))
                .map(|(start, end)| {
                    Diagnostic::new_simple(Range::new(start, end), format!("{:?}", err))
                });
            if let Some(diag) = diag {
                diagnostics.push(diag);
            }
        }
        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, params.version)
            .await;
    }
}

const VERSION_STRING: &'static str = env!("VERSION_STRING");

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-v", "--version"]) {
        println!("version: {}", VERSION_STRING);
        return;
    }

    if args.contains(["-h", "--help"]) {
        println!("{}", HELP);
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        ast_map: DashMap::new(),
        document_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_parses_simple_hurl_document() {
        let (ast, errs) = hurl_parser::parser::parser::parse_ast("GET {{abc}}");
        assert!(ast != None);
        assert!(errs.len() == 0);
    }
}
