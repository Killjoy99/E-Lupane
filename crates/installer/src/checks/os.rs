// check which OS we are running on
mod utils;
use utils::OperatingSystem;

pub fn current_os() -> OperatingSystem {
    match std::env::consts::OS {
        "windows" => OperatingSystem::Windows,
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        _ => OperatingSystem::Unknown,
    }
}