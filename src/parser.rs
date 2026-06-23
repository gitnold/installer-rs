use crate::installer;
use crate::lexer::{PackageManager, Token, Tokens};
use crate::print_message;
use crate::utils::config_parser::Config;
use crate::utils::runner::{Runner, SystemRunner};
use crate::utils;

pub struct Parser {
    tokens: Tokens,
    install_ctx: InstallCtx,
    pkg_ctx: PackageCtx,
    config: CmdConfig,
}

#[derive(Debug, PartialEq)]
struct CmdConfig {
    pkg_man: String,
    pkg: String,
    custom_cmd: Option<String>,
    install: bool,
    add_pkg: bool,
}

impl CmdConfig {
    fn new() -> Self {
        Self {
            pkg_man: String::new(),
            pkg: String::new(),
            custom_cmd: None,
            install: false,
            add_pkg: false,
        }
    }
}

struct InstallCtx {
    pkgman_set: bool,
    pkg_set: bool,
}
impl InstallCtx {
    fn new() -> Self {
        Self {
            pkgman_set: false,
            pkg_set: false,
        }
    }

    fn is_true(&self) -> bool {
        self.pkgman_set && self.pkg_set
    }
}

struct PackageCtx {
    pkgman_set: bool,
}
impl PackageCtx {
    fn is_true(&self) -> bool {
        self.pkgman_set
    }
}

impl Parser {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens,
            install_ctx: InstallCtx::new(),
            pkg_ctx: PackageCtx { pkgman_set: false },
            config: CmdConfig::new(),
        }
    }

    pub fn evaluate(&self, json_config: Option<&mut Config>) {
        self.evaluate_with_runner(json_config, &SystemRunner);
    }

    pub fn evaluate_with_runner(&self, mut json_config: Option<&mut Config>, runner: &dyn Runner) {
        // Case 1: CLI specifies both -p and -a with -i → single package install
        if self.config.install && self.install_ctx.is_true() {
            match runner.run_install(&self.config.pkg_man, &[&self.config.pkg]) {
                Ok(status) => {
                    if status.success() {
                        print_message!(info, "Command run successfully!!");
                    } else {
                        print_message!(error, "Command exited with status: {status}");
                    }
                }
                Err(e) => {
                    print_message!(error, "Command run failed: {e}");
                }
            }
        // Case 2: -i alone, or -i with only -p → install from config
        } else if self.config.install {
            if let Some(ref config) = json_config {
                let executor = installer::Executor::new(config);
                executor.install_all_with_runner(runner);
            } else {
                print_message!(error, "No config file provided for installation");
            }
        }

        // Run custom CLI command (independent of install)
        if let Some(cmd) = &self.config.custom_cmd {
            if let Err(e) = runner.run_shell(cmd) {
                eprintln!("Custom command failed: {e}");
            }
        }

        // Add package to config
        if self.config.add_pkg && self.pkg_ctx.is_true() {
            if let Some(ref mut json_conf) = json_config {
                json_conf.add_package(&self.config.pkg, &self.config.pkg_man);
            }
        }
    }

    pub fn parse(&mut self) -> &mut Self {
        for option in &self.tokens.options {
            match option {
                Token::Install => self.config.install = true,

                Token::SelfUpdate => installer::Executor::self_install(),

                Token::PackageManager(s) => match s {
                    PackageManager::Dnf => {
                        self.config.pkg_man = "dnf".to_string();
                        self.install_ctx.pkgman_set = true;
                        self.pkg_ctx.pkgman_set = true;
                    }
                    PackageManager::Apt => {
                        self.config.pkg_man = "apt".to_string();
                        self.install_ctx.pkgman_set = true;
                        self.pkg_ctx.pkgman_set = true;
                    }
                    PackageManager::Git => {
                        self.config.pkg_man = "git".to_string();
                        self.install_ctx.pkgman_set = true;
                        self.pkg_ctx.pkgman_set = true;
                    }
                    PackageManager::CustomCmd => {
                        self.config.pkg_man = "custom".to_string();
                        self.install_ctx.pkgman_set = true;
                        self.pkg_ctx.pkgman_set = true;
                    }
                    PackageManager::Illegal(value) => {
                        println!("Illegal package source specified => {value}");
                        utils::print_help();
                        break;
                    }
                },
                Token::Illegal(value) => {
                    println!("Illegal option encountered! => {value}");
                    utils::print_help();
                    break;
                }
                Token::AddPackage(s) => {
                    self.config.add_pkg = true;
                    self.install_ctx.pkg_set = true;
                    self.pkg_ctx.pkgman_set = true;
                    self.config.pkg = s.clone();
                }
                Token::CustomCmd(s) => {
                    self.config.custom_cmd = Some(s.clone());
                }
                Token::Help => {
                    println!("Printing Help.....");
                    utils::print_help();
                    break;
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::runner::test_util::MockRunner;

    fn tokens_from(args: &[&str]) -> Tokens {
        // Prepend a dummy program name since from_strs skips argv[0]
        let mut full = vec!["opencode"];
        full.extend_from_slice(args);
        let args: Vec<String> = full.iter().map(|s| s.to_string()).collect();
        Tokens::from_strs(args)
    }

    fn config_with_apt_pkg() -> Config {
        use crate::utils::config_parser::{PackageManager as ConfigPkgMan, RepoPackage};
        let mut pkg_managers = std::collections::HashMap::new();
        pkg_managers.insert(
            "apt".to_string(),
            ConfigPkgMan {
                os: "debian".to_string(),
                packages: vec![
                    RepoPackage { name: "zoxide".into(), version: None, repo_name: "zoxide".into() },
                    RepoPackage { name: "python".into(), version: None, repo_name: "python3".into() },
                ],
            },
        );
        Config {
            package_managers: pkg_managers,
            custom_packages: vec![],
        }
    }

    #[test]
    fn test_parse_single_install() {
        let lexer = tokens_from(&["-p", "dnf", "-a", "zoxide", "-i"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        assert!(parser.config.install);
        assert!(parser.install_ctx.is_true());
        assert_eq!(parser.config.pkg_man, "dnf");
        assert_eq!(parser.config.pkg, "zoxide");
        // -a side-effect: add_pkg is also set
        assert!(parser.config.add_pkg);
        assert!(parser.config.custom_cmd.is_none());
    }

    #[test]
    fn test_parse_install_only_triggers_config_install() {
        let lexer = tokens_from(&["-i"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        assert!(parser.config.install);
        assert!(!parser.install_ctx.is_true()); // no -p or -a
    }

    #[test]
    fn test_parse_add_package() {
        let lexer = tokens_from(&["-p", "apt", "-a", "zoxide"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        assert!(parser.config.add_pkg);
        assert!(parser.pkg_ctx.is_true());
        assert_eq!(parser.config.pkg, "zoxide");
        assert_eq!(parser.config.pkg_man, "apt");
    }

    #[test]
    fn test_parse_custom_cmd() {
        let lexer = tokens_from(&["-c", "echo hello"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        assert_eq!(parser.config.custom_cmd, Some("echo hello".to_string()));
    }

    #[test]
    fn test_evaluate_install_with_specific_pkgman_and_pkg() {
        let lexer = tokens_from(&["-p", "dnf", "-a", "zoxide", "-i"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        let mock = MockRunner::new();
        parser.evaluate_with_runner(None, &mock);

        let installs = mock.install_calls.lock().unwrap();
        assert_eq!(installs.len(), 1);
        assert_eq!(installs[0].0, "dnf");
        assert_eq!(installs[0].1, vec!["zoxide"]);
    }

    #[test]
    fn test_evaluate_install_only_runs_config_install() {
        let lexer = tokens_from(&["-i"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        let mock = MockRunner::new();
        let mut config = config_with_apt_pkg();
        parser.evaluate_with_runner(Some(&mut config), &mock);

        let installs = mock.install_calls.lock().unwrap();
        assert_eq!(installs.len(), 1);
        assert_eq!(installs[0].0, "apt");
        assert_eq!(installs[0].1, vec!["zoxide", "python3"]);
    }

    #[test]
    fn test_evaluate_custom_cmd_runs_after_install() {
        let lexer = tokens_from(&["-i", "-c", "echo done"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        let mock = MockRunner::new();
        let mut config = config_with_apt_pkg();
        parser.evaluate_with_runner(Some(&mut config), &mock);

        // Should run both install (from config) AND custom cmd
        let installs = mock.install_calls.lock().unwrap();
        let shells = mock.shell_calls.lock().unwrap();
        assert_eq!(installs.len(), 1, "should install from config");
        assert_eq!(shells.len(), 1, "should also run custom cmd");
        assert_eq!(shells[0], "echo done");
    }

    #[test]
    fn test_evaluate_custom_cmd_only() {
        let lexer = tokens_from(&["-c", "echo hello"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        let mock = MockRunner::new();
        parser.evaluate_with_runner(None, &mock);

        let installs = mock.install_calls.lock().unwrap();
        let shells = mock.shell_calls.lock().unwrap();
        assert!(installs.is_empty());
        assert_eq!(shells.len(), 1);
        assert_eq!(shells[0], "echo hello");
    }

    #[test]
    fn test_evaluate_add_package() {
        let lexer = tokens_from(&["-p", "apt", "-a", "neovim"]);
        let mut parser = Parser::new(lexer);
        parser.parse();

        let mut config = config_with_apt_pkg();
        let mock = MockRunner::new();
        parser.evaluate_with_runner(Some(&mut config), &mock);

        assert!(config.package_managers.contains_key("apt"));
        let apt = config.package_managers.get("apt").unwrap();
        let names: Vec<&str> = apt.packages.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"neovim"), "neovim should be added to apt packages");
    }

    #[test]
    fn test_evaluate_illegal_pkg_manager_prints_help() {
        let lexer = tokens_from(&["-p", "brew"]);
        let mut parser = Parser::new(lexer);
        parser.parse();
        // Should have broken out of loop with Illegal token still in stream
    }

    #[test]
    fn test_parse_help_breaks() {
        let lexer = tokens_from(&["-h", "-i"]);
        let mut parser = Parser::new(lexer);
        parser.parse();
        // Help causes break before -i is processed
        assert!(!parser.config.install);
    }
}
