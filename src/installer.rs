use crate::utils::config_parser::Config;
use crate::utils::constants;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;


pub fn install(config: &Config, pkg_man: &str) {
    let Some(pkgman) = config.package_managers.get(pkg_man) else {
        eprintln!("Package manager '{pkg_man}' not found in config");
        return;
    };

    let packages: Vec<&str> = pkgman.packages.iter().map(|p| p.repo_name.as_str()).collect();
    if packages.is_empty() {
        return;
    }

    let status = Command::new("sudo")
        .arg(pkg_man)
        .arg("install")
        .args(&packages)
        .status();

    match status {
        Ok(s) if s.success() => println!("Installation completed successfully."),
        Ok(s) => eprintln!("Installation failed with status: {s}"),
        Err(e) => eprintln!("Failed to execute installation: {e}"),
    }
}
pub fn self_install() {
    let orig_dir = env::current_dir().ok();
    let install_path = Path::new(constants::INSTALL_PATH);

    if env::set_current_dir(install_path).is_err() {
        eprintln!("Failed to change to directory: {}", constants::INSTALL_PATH);
        return;
    }

    // Pull latest changes; abort on failure.
    if let Err(e) = Command::new("git")
        .arg("pull")
        .status()
        .map(|s| if !s.success() {
            eprintln!("git pull exited with non-zero status: {s}");
        })
    {
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

    // Copy the built binary (not the source directory).
    let binary_path = install_path.join("target/release/mig");
    let dest = Path::new(constants::DEST_PATH).join("mig");
    if let Err(error) = fs::copy(&binary_path, &dest) {
        eprintln!(
            "Failed to copy binary from {} to {}: {error}",
            binary_path.display(),
            dest.display()
        );
    }

    // Restore original working directory.
    let _ = orig_dir.map(env::set_current_dir);
}
