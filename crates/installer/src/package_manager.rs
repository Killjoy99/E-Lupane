use std::process::Command;


#[derive(Debug, Clone, Copy)]
pub enum PackageManager{
    Apt,
    Dnf,
    Yum,
    Pacman,
    Brew,
    Winget,
    Chocolatey,
    Unknown
}

pub fn detect_package_manager() -> PackageManager {
    if command_exists("apt") {
        return PackageManager::Apt;
    }

    if command_exists("dnf") {
        return PackageManager::Dnf;
    }

    if command_exists("yum") {
        return PackageManager::Yum;
    }

    if command_exists("pacman") {
        return PackageManager::Pacman;
    }

    if command_exists("brew") {
        return PackageManager::Brew;
    }

    if command_exists("winget") {
        return PackageManager::Winget;
    }

    if command_exists("choco") {
        return PackageManager::Chocolatey;
    }

    PackageManager::Unknown
}

/// The package manager that provisions the *runtime* environment.
///
/// On a Windows host the runtime environment is the WSL distro (Debian-family:
/// Kali/Ubuntu/Debian), so `apt` is the manager that matters there regardless
/// of whatever Windows package manager the host happens to have.
pub fn effective_manager(host: PackageManager) -> PackageManager {
    if cfg!(target_os = "windows") {
        PackageManager::Apt
    } else {
        host
    }
}

fn command_exists(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    let output = Command::new("where")
        .arg(command)
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("which")
        .arg(command)
        .output();

    output
        .map(|o| o.status.success())
        .unwrap_or(false)
}