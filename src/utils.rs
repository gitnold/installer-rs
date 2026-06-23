
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

    // Compute column widths from the widest value in each column (including header)
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
        pub fn add_package(&mut self, package: &str, pkg_man: &str)  {
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

        // OS-specific repository name
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

    /// Read the OS identifier from `/etc/os-release`.
    pub fn get_os() -> String {
        let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
        for line in content.lines() {
            if line.starts_with("ID=") {
                return line.replace("ID=", "").replace('"', "").to_string();
            }
        }
        String::new()
    }


    pub fn run_cmd(pkg_man: &str, package: &str) -> Result<ExitStatus, Error> {
        Command::new(pkg_man).arg("install").arg(package).status()
    }

    /// Execute an arbitrary shell command via `sh -c`.
    pub fn run_custom(cmd: &str) -> Result<ExitStatus, Error> {
        //TODO: check if code below has any potential security risks
        Command::new("sh").arg("-c").arg(cmd).status()
    }
}


pub mod constants {
    pub const URL: &str = "https://github.com/";
    // FIX: check the git clone folder location, specify explicitly or rename the repo to mig
    pub const INSTALL_PATH: &str = "~/Applications/mig/";
    pub const DEST_PATH: &str = "~/.local/bin/";
    pub const CONFIG_PATH: &str = "~/Applications/installer-rs/";
}
