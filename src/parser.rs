use crate::installer;
use crate::lexer::{PackageManager, Token, Tokens};
use crate::print_message;
use crate::utils::config_parser::Config;
use crate::utils::{self, runner};

pub struct Parser {
    tokens: Tokens,
    install_ctx: InstallCtx,
    pkg_ctx: PackageCtx,
    customcmd_ctx: CustomCmdCtx,
    config: CmdConfig,
}

/// Aggregated configuration derived from parsed tokens.
///
/// # Bug fix: removed unused `config_path` field
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

/// Tracks whether enough context has been provided for an `-i` (install) operation.
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

/// Tracks whether a package manager has been selected.
struct PackageCtx {
    pkgman_set: bool,
}
impl PackageCtx {
    fn is_true(&self) -> bool {
        self.pkgman_set
    }
}

/// Tracks whether a custom command has been provided.
struct CustomCmdCtx {
    package_set: bool,
}
impl CustomCmdCtx {
    fn is_true(&self) -> bool {
        self.package_set
    }
}

impl Parser {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens,
            install_ctx: InstallCtx::new(),
            pkg_ctx: PackageCtx { pkgman_set: false },
            customcmd_ctx: CustomCmdCtx { package_set: false },
            config: CmdConfig::new(),
        }
    }

    pub fn evaluate(&self, json_config: Option<&mut Config>) {
        if self.customcmd_ctx.is_true() {
            if let Some(cmd) = &self.config.custom_cmd {
                if let Err(e) = runner::run_custom(cmd) {
                    eprintln!("Custom command failed: {e}");
                }
            }
            return;
        }
        if self.config.install && self.install_ctx.is_true() {
            match runner::run_cmd(self.config.pkg_man.as_str(), self.config.pkg.as_str()) {
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
        } else if self.config.add_pkg && self.pkg_ctx.is_true() {
            if let Some(json_conf) = json_config {
                json_conf.add_package(&self.config.pkg, &self.config.pkg_man);
            }
        }
    }


    pub fn parse(&mut self) -> &mut Self {
        for option in &self.tokens.options {
            match option {
                Token::Install => self.config.install = true,

                Token::SelfUpdate => installer::self_install(),

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
                    self.customcmd_ctx.package_set = true;
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
