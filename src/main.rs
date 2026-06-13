mod installer;
mod lexer;
mod parser;
mod terminal;
mod utils;

fn main() -> Result<(), serde_json::Error> {
    let args: Vec<String> = std::env::args().collect();
    let cmd_lexer = lexer::Tokens::from_strs(args);
    let mut cmd_parser = parser::Parser::new(cmd_lexer);

    // parse the cmd tokens.
    cmd_parser.parse();

    // load config and evaluate.
    let mut config = utils::config_parser::Config::from_file("./assets/format1.json")?;
    cmd_parser.evaluate(Some(&mut config));

    Ok(())
}
