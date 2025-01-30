# hurl-language-server

A language server for hurl files.

# ~~Features~~ Roadmap

- [ ] Support full hurl syntax up to Hurl version 6.0 (requirement before leaving version releasing version 0.1.0)
- [ ] Completion
  - [x] Initial dumb completion for keywords
  - [ ] Completion documentation (Same as hover documentation)
  - [ ] Context aware completion using AST
  - [ ] Completion for API specs and use
    - [ ] Use a configuration file to specify the file or url of an API spec
    - [ ] Completion for APIs adhering to the [OpenAPI](https://spec.openapis.org/) spec
    - [ ] Completion for GraphQL APIs
    - [ ] Completion for Soap APIs using WSDL files.
  - [ ] Snippets
    - [x] Simple snippets for entries
    - [ ] Snippets for common scenarios like (csrf token, oauth, default capture values, etc)
- [ ] Diagnostics
  - [x] Diagnostic errors work for currently implemented portion of the Hurl version 6.0 grammer
  - [ ] Human readable error messages
  - [ ] Type checking in asserts
- [ ] Hover Documentation
  - [ ] HTTP keywords
  - [ ] Hurl keywords
  - [ ] Request field documentation (from API spec files)
- [ ] Go to Definition
  - [ ] Variables should go to last capture location for that variable or the value in the vars.env file
- [ ] Find References
  - [ ] Find variable references
  - [ ] Find request field references
- [ ] Renaming
  - [ ] Rename capture variables
  - [ ] Rename field names in jsonpath queries
  - [ ] Rename field names in json fields
  - [ ] Rename xml elements
- [ ] Semantic Tokens
- [ ] Folding
  - [ ] Fold entries
  - [ ] Fold sections
  - [ ] Fold JSON objects/arrays
  - [ ] Fold XML tags
- [ ] Code Lens
  - [ ] "Run | Run With Vars" similar to rust-analyzer's code lens for "Run" and "Debug".
        External plugins will implement how to do these actions
- [ ] Code Actions
  - [ ] Run file (External plugins will implement how to do these actions)
  - [ ] Run file with vars (External plugins will implement how to do these actions)
  - [ ] Run file with varfile (External plugins will implement how to do these actions)
  - [ ] Extract to Variable (Replace a value with a template and add a variable option to the entry)
  - [ ] Move Variable to varfile
  - [ ] Inline variable option
- [ ] Formatting
- [ ] Document Link
  - [ ] Links to external documentation (similar feature to gopls)
