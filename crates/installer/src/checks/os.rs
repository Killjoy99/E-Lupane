// Check which OS we are running on.
use crate::checks::utils::OperatingSystem;

pub fn current_os() -> OperatingSystem {
    match std::env::consts::OS {
        "windows" => OperatingSystem::Windows,
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        _ => OperatingSystem::Unknown,
    }
}