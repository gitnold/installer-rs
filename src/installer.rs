use crate::utils::config_parser::Config;
use crate::utils::constants;
use crate::utils::runner::{Runner, SystemRunner};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Executor<'a> {
    config: &'a Config,
}

#[allow(dead_code)]
impl<'a> Executor<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn install_all(&self) {
        self.install_all_with_runner(&SystemRunner);
    }

    pub fn install_all_with_runner(&self, runner: &dyn Runner) {
        for (pkg_man, pm) in &self.config.package_managers {
            let packages: Vec<&str> = pm.packages.iter().map(|p| p.repo_name.as_str()).collect();
            if packages.is_empty() {
                continue;
            }
            match runner.run_install(pkg_man, &packages) {
                Ok(s) if s.success() => println!("{pkg_man} installation completed."),
                Ok(s) => eprintln!("{pkg_man} installation failed with status: {s}"),
                Err(e) => eprintln!("{pkg_man} installation failed: {e}"),
            }
        }
        self.install_custom_packages_with_runner(runner);
    }

    pub fn install_pkg_man(&self, pkg_man: &str) {
        let Some(pm) = self.config.package_managers.get(pkg_man) else {
            eprintln!("Package manager '{pkg_man}' not found in config");
            return;
        };
        let packages: Vec<&str> = pm.packages.iter().map(|p| p.repo_name.as_str()).collect();
        if packages.is_empty() {
            return;
        }
        let runner = SystemRunner;
        match runner.run_install(pkg_man, &packages) {
            Ok(s) if s.success() => println!("{pkg_man} installation completed."),
            Ok(s) => eprintln!("{pkg_man} installation failed with status: {s}"),
            Err(e) => eprintln!("{pkg_man} installation failed: {e}"),
        }
    }

    pub fn install_custom_packages(&self) {
        self.install_custom_packages_with_runner(&SystemRunner);
    }

    pub fn install_custom_packages_with_runner(&self, runner: &dyn Runner) {
        for pkg in &self.config.custom_packages {
            println!("Installing custom package: {}", pkg.name);
            for (i, step) in pkg.install_steps.iter().enumerate() {
                println!("  Step {}/{}: {step}", i + 1, pkg.install_steps.len());
                if let Err(e) = runner.run_shell(step) {
                    eprintln!("  Install step failed: {e}");
                    break;
                }
            }
            for cmd in &pkg.post_install {
                println!("  Post-install: {cmd}");
                if let Err(e) = runner.run_shell(cmd) {
                    eprintln!("  Post-install command failed: {e}");
                }
            }
        }
    }

    pub fn self_install() {
        let orig_dir = env::current_dir().ok();
        let install_path = Path::new(constants::INSTALL_PATH);

        if env::set_current_dir(install_path).is_err() {
            eprintln!("Failed to change to directory: {}", constants::INSTALL_PATH);
            return;
        }

        if let Err(e) = Command::new("git").arg("pull").status().map(|s| {
            if !s.success() {
                eprintln!("git pull exited with non-zero status: {s}");
            }
        }) {
            eprintln!("Failed to run git pull: {e}");
            let _ = orig_dir.map(env::set_current_dir);
            return;
        }

        match Command::new("cargo").args(["build", "--release"]).status() {
            Ok(s) if s.success() => println!("Binary successfully built."),
            Ok(s) => {
                eprintln!("cargo build failed with status: {s}");
                let _ = orig_dir.map(env::set_current_dir);
                return;
            }
            Err(e) => {
                eprintln!("cargo build failed with error: {e}");
                let _ = orig_dir.map(env::set_current_dir);
                return;
            }
        }

        let binary_path = install_path.join("target/release/mig");
        let dest = Path::new(constants::DEST_PATH).join("mig");
        if let Err(error) = fs::copy(&binary_path, &dest) {
            eprintln!(
                "Failed to copy binary from {} to {}: {error}",
                binary_path.display(),
                dest.display()
            );
        }

        let _ = orig_dir.map(env::set_current_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::runner::test_util::MockRunner;
    use crate::utils::config_parser::{CustomPackage, PackageManager as ConfigPkgMan, RepoPackage};
    use std::collections::HashMap;

    fn sample_config() -> Config {
        let mut package_managers = HashMap::new();
        package_managers.insert(
            "apt".to_string(),
            ConfigPkgMan {
                os: "debian".to_string(),
                packages: vec![
                    RepoPackage { name: "zoxide".into(), version: None, repo_name: "zoxide".into() },
                ],
            },
        );
        Config {
            package_managers,
            custom_packages: vec![
                CustomPackage {
                    name: "neovim".into(),
                    version: Some("latest".into()),
                    install_steps: vec!["git clone https://github.com/neovim/neovim".into()],
                    post_install: vec!["nvim --version".into()],
                },
            ],
        }
    }

    #[test]
    fn test_install_all_calls_runner_for_each_pkg_man() {
        let config = sample_config();
        let executor = Executor::new(&config);
        let mock = MockRunner::new();
        executor.install_all_with_runner(&mock);

        let installs = mock.install_calls.lock().unwrap();
        assert_eq!(installs.len(), 1);
        assert_eq!(installs[0].0, "apt");
        assert_eq!(installs[0].1, vec!["zoxide"]);
    }

    #[test]
    fn test_install_custom_packages_calls_shell_for_steps_and_post() {
        let config = sample_config();
        let executor = Executor::new(&config);
        let mock = MockRunner::new();
        executor.install_custom_packages_with_runner(&mock);

        let shells = mock.shell_calls.lock().unwrap();
        assert_eq!(shells.len(), 2);
        assert!(shells[0].contains("git clone"));
        assert_eq!(shells[1], "nvim --version");
    }

    #[test]
    fn test_install_empty_config_does_nothing() {
        let config = Config::new();
        let executor = Executor::new(&config);
        let mock = MockRunner::new();
        executor.install_all_with_runner(&mock);

        let installs = mock.install_calls.lock().unwrap();
        let shells = mock.shell_calls.lock().unwrap();
        assert!(installs.is_empty());
        assert!(shells.is_empty());
    }
}
