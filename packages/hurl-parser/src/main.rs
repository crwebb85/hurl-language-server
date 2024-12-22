mod parser;
// mod test;

#[derive(Debug)]
pub struct ImCompleteSemanticToken {
    pub start: usize,
    pub length: usize,
    pub token_type: usize,
}

fn main() {
    println!("hi");

    //For quick debugging
    // let dummy_parser = ast_parser();

    // let src = "GET\nhttps://example.org";
    // let ast_result = dummy_parser.parse(src);
    // println!("{:?}", ast_result);
}
