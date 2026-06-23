
// letters in use -a, -h, -u, -p, -i, -c, v
pub fn print_help() {
    let options = [
        ("-h", "-h",                           "Print the help message"),
        ("-a", "-a <package name>",             "Add a package to the config / package database"),
        ("-u", "-u",                            "Update the installer to the newest version"),
        ("-p", "-p <package manager>",          "Specifies the package manager to be used"),
        ("-i", "-i or -i <package>",            "Install packages listed in the config files; passing a value installs that package only"),
        ("-c", "-c \"echo 'me' >> ~/.bashrc\"", "Run a command after the specified operation"),
    ];

    let w0 = options.iter().map(|(f, _, _)| f.len()).max().unwrap_or(0).max("Flag".len());
    let w1 = options.iter().map(|(_, u, _)| u.len()).max().unwrap_or(0).max("Usage".len());
    let w2 = options.iter().map(|(_, _, d)| d.len()).max().unwrap_or(0).max("Description".len());

    let sep = format!(
        "+-{}-+-{}-+-{}-+",
        "-".repeat(w0),
        "-".repeat(w1),
        "-".repeat(w2),
    );

    println!("Welcome to the backup installer ^_^\n");
    println!("{sep}");
    println!("| {:<w0$} | {:<w1$} | {:<w2$} |", "Flag", "Usage", "Description");
    println!("{sep}");

    for (flag, usage, desc) in &options {
        println!("| {:<w0$} | {:<w1$} | {:<w2$} |", flag, usage, desc);
    }

    println!("{sep}");
}

pub mod config_parser {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;

    use crate::utils::runner::get_os;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Config {
        pub package_managers: HashMap<String, PackageManager>,

        #[serde(default)]
        pub custom_packages: Vec<CustomPackage>,
    }

    #[allow(dead_code)]
    impl Config {
        pub fn new() -> Self {
            Self {
                package_managers: HashMap::new(),
                custom_packages: Vec::new(),
            }
        }

        pub fn from_json_str(json_str: String) -> Result<Config, serde_json::Error> {
            let config: Config = serde_json::from_str(&json_str)?;
            Ok(config)
        }

        pub fn from_file(path: &str) -> Result<Config, serde_json::Error> {
            let json_contents =
                std::fs::read_to_string(path).map_err(|e| {
                    serde_json::Error::io(e).into()
                })?;
            serde_json::from_str(&json_contents)
        }

        pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string_pretty(self)
        }

        pub fn write_to_file(&self, path: &str) -> Result<bool, serde_json::Error> {
            let string_rep = serde_json::to_string_pretty(self)?;
            let mut file = File::create(path).map_err(|e| serde_json::Error::io(e).into())?;
            file.write(string_rep.as_bytes())
                .map_err(|e| serde_json::Error::io(e).into())?;
            Ok(true)
        }

        pub fn add_package(&mut self, package: &str, pkg_man: &str) {
            let pkgman_packages = self.package_managers
                .entry(pkg_man.to_string())
                .or_insert(PackageManager{os: get_os(), packages: Vec::new()});
            pkgman_packages.packages.push(RepoPackage{name: package.to_string(), version: None, repo_name: package.to_string()});
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PackageManager {
        pub os: String,
        pub packages: Vec<RepoPackage>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RepoPackage {
        pub name: String,

        #[serde(default)]
        pub version: Option<String>,

        pub repo_name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CustomPackage {
        pub name: String,

        #[serde(default)]
        pub version: Option<String>,

        pub install_steps: Vec<String>,

        #[serde(default)]
        pub post_install: Vec<String>,
    }
}

pub mod runner {
    use std::{
        fs,
        io::Error,
        process::{Command, ExitStatus},
    };

    pub trait Runner {
        fn run_install(&self, pkg_man: &str, packages: &[&str]) -> Result<ExitStatus, Error>;
        fn run_shell(&self, cmd: &str) -> Result<ExitStatus, Error>;
    }

    pub struct SystemRunner;
    impl Runner for SystemRunner {
        fn run_install(&self, pkg_man: &str, packages: &[&str]) -> Result<ExitStatus, Error> {
            let mut cmd = Command::new("sudo");
            cmd.arg(pkg_man).arg("install");
            for pkg in packages {
                cmd.arg(pkg);
            }
            cmd.status()
        }

        fn run_shell(&self, cmd: &str) -> Result<ExitStatus, Error> {
            Command::new("sh").arg("-c").arg(cmd).status()
        }
    }

    #[cfg(test)]
    pub mod test_util {
        use super::*;
        use std::sync::Mutex;

        pub struct MockRunner {
            pub install_calls: Mutex<Vec<(String, Vec<String>)>>,
            pub shell_calls: Mutex<Vec<String>>,
        }

        impl MockRunner {
            pub fn new() -> Self {
                Self {
                    install_calls: Mutex::new(Vec::new()),
                    shell_calls: Mutex::new(Vec::new()),
                }
            }
        }

        impl Runner for MockRunner {
            fn run_install(&self, pkg_man: &str, packages: &[&str]) -> Result<ExitStatus, Error> {
                let pkgs: Vec<String> = packages.iter().map(|s| s.to_string()).collect();
                self.install_calls.lock().unwrap().push((pkg_man.to_string(), pkgs));
                // Return a fake success status
                unsafe {
                    Ok(std::mem::transmute::<i32, ExitStatus>(0))
                }
            }

            fn run_shell(&self, cmd: &str) -> Result<ExitStatus, Error> {
                self.shell_calls.lock().unwrap().push(cmd.to_string());
                unsafe {
                    Ok(std::mem::transmute::<i32, ExitStatus>(0))
                }
            }
        }
    }

    pub fn get_os() -> String {
        let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
        for line in content.lines() {
            if line.starts_with("ID=") {
                return line.replace("ID=", "").replace('"', "").to_string();
            }
        }
        String::new()
    }

    #[allow(dead_code)]
    pub fn run_cmd(pkg_man: &str, package: &str) -> Result<ExitStatus, Error> {
        Command::new("sudo").arg(pkg_man).arg("install").arg(package).status()
    }

    #[allow(dead_code)]
    pub fn run_custom(cmd: &str) -> Result<ExitStatus, Error> {
        Command::new("sh").arg("-c").arg(cmd).status()
    }
}

pub mod constants {
    #[allow(dead_code)]
    pub const URL: &str = "https://github.com/";
    pub const INSTALL_PATH: &str = "~/Applications/mig/";
    pub const DEST_PATH: &str = "~/.local/bin/";
    #[allow(dead_code)]
    pub const CONFIG_PATH: &str = "~/Applications/installer-rs/";
}
