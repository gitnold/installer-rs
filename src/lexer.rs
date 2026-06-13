/// Supported package manager backends (apt, dnf, git, custom commands).
#[derive(Debug)]
pub enum PackageManager {
    Dnf,
    Apt,
    Git,
    CustomCmd,
    Illegal(String),
}

/// Token types produced by the lexer from raw CLI arguments.
#[derive(Debug)]
pub enum Token {
    Install,
    PackageManager(PackageManager),
    AddPackage(String),
    CustomCmd(String),
    Help,
    SelfUpdate,
    Illegal(String),
}

/// A list of tokens parsed from the command line.
#[derive(Debug)]
pub struct Tokens {
    pub options: Vec<Token>,
}

impl Tokens {
    pub fn new(args: Vec<Token>) -> Self {
        Self { options: args }
    }

    /// Convert raw argument strings into a list of tokens.
    ///
    /// # Bugs fixed
    ///
    /// - **Index never advanced**: the old code used `let index = 0` (immutable) and
    ///   `index + 1;` (a discarded no-op), causing every flag to read `args[1]`.
    ///   The loop would never terminate on unexpected input and could index out of
    ///   bounds.
    /// - **Out-of-bounds on value-bearing flags**: `-a`, `-p`, `-c` always read
    ///   `args[index + 1]` without checking whether that index exists.
    /// - **Empty `Illegal` string**: the catch-all `_` branch stored `String::from("")`,
    ///   making error messages useless.
    /// - **Program name included**: the caller previously passed `env::args().collect()`
    ///   which includes `argv[0]`; we now skip the first element.
    pub fn from_strs(args: Vec<String>) -> Self {

        // NOTE: lexer doesn't check whether next token is a possible option for args having values.
        // only checks bounds and reads the next value as a value for the current token.
        let mut options: Vec<Token> = Vec::new();
        // skip the program name (argv[0])
        let args = if args.is_empty() {
            &[] as &[String]
        } else {
            &args[1..]
        };
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "-a" => {
                    i += 1;
                    if i >= args.len() {
                        options.push(Token::Illegal(String::from(
                            "-a requires a package name",
                        )));
                        break;
                    }
                    options.push(Token::AddPackage(args[i].clone()));
                }
                "-h" => options.push(Token::Help),
                "-u" => options.push(Token::SelfUpdate),
                "-p" => {
                    i += 1;
                    if i >= args.len() {
                        options.push(Token::Illegal(String::from(
                            "-p requires a package manager name",
                        )));
                        break;
                    }
                    options.push(Token::PackageManager(match args[i].as_str() {
                        "dnf" => PackageManager::Dnf,
                        "apt" => PackageManager::Apt,
                        "git" => PackageManager::Git,
                        "custom" => PackageManager::CustomCmd,
                        value => PackageManager::Illegal(format!("unknown or unsupported package source: {value}")),
                    }));
                }
                "-i" => options.push(Token::Install),
                "-c" => {
                    i += 1;
                    if i >= args.len() {
                        options.push(Token::Illegal(String::from(
                            "-c requires a command string",
                        )));
                        break;
                    }
                    // TODO: ensure multiword or quoted arguments are interpreted correctly.
                    options.push(Token::CustomCmd(args[i].clone()));
                }
                other => options.push(Token::Illegal(format!("unknown flag: {other}"))),
            }
            i += 1;
        }

        Self { options }
    }
}
