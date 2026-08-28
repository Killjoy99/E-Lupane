use std::process::Command;

use crate::package_manager::PackageManager;

pub fn install(
    manager: PackageManager,
) -> Result<(), Box<dyn std::error::Error>> {
    match manager {
        PackageManager::Apt => {
            install_apt()
        }

        PackageManager::Brew => {
            install_brew()
        }

        _ => Err(
            "Unsupported package manager"
                .into(),
        ),
    }
}

fn install_apt(
) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("sudo")
        .args(["apt", "update"])
        .status()?;

    let status = Command::new("sudo")
        .args([
            "apt",
            "install",
            "-y",
            "redis-server",
        ])
        .status()?;

    if !status.success() {
        return Err(
            "Failed to install Redis".into(),
        );
    }

    Ok(())
}

fn install_brew(
) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("brew")
        .args(["install", "redis"])
        .status()?;

    if !status.success() {
        return Err(
            "Failed to install Redis".into(),
        );
    }

    Ok(())
}