/// Supported package manager backends (apt, dnf, git, custom commands).
#[derive(Debug, PartialEq)]
pub enum PackageManager {
    Dnf,
    Apt,
    Git,
    CustomCmd,
    Illegal(String),
}

/// Token types produced by the lexer from raw CLI arguments.
#[derive(Debug, PartialEq)]
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

impl PartialEq for Tokens {
    fn eq(&self, other: &Tokens) -> bool {
        let mut equal = true;
        let mut index = 0;
        let mut count_equal = 0;

        if self.options.len() != other.options.len() {
            return false;
        }

        for option in &self.options {
            if *option == other.options[index] {
                count_equal += 1;
            } else {
                equal = false;
                break;
            }
            index += 1;
        }

        if count_equal == self.options.len() {
            equal = true;
        }

        equal
    }
}

impl Tokens {
    #[allow(dead_code)]
    pub fn new(args: Vec<Token>) -> Self {
        Self { options: args }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_flags() {
        // from_strs skips argv[0] (the program name), so prepend a dummy
        let test_strings = vec!["prog", "-p", "dnf", "-a", "zoxide", "-i", "-u", "-c", "echo $SHELL"];
        let tokens = vec![
            Token::PackageManager(PackageManager::Dnf),
            Token::AddPackage(String::from("zoxide")),
            Token::Install,
            Token::SelfUpdate,
            Token::CustomCmd(String::from("echo $SHELL"))
        ];

        let args = test_strings.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let lexer = Tokens::from_strs(args);

        assert_eq!(lexer.options, tokens);
    }

    #[test]
    fn test_parse_help_flag() {
        let test_strings = vec!["prog", "-h"];
        let args = test_strings.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let lexer = Tokens::from_strs(args);
        assert_eq!(lexer.options, vec![Token::Help]);
    }

    #[test]
    fn test_parse_install_only() {
        let test_strings = vec!["prog", "-i"];
        let args = test_strings.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let lexer = Tokens::from_strs(args);
        assert_eq!(lexer.options, vec![Token::Install]);
    }
}
