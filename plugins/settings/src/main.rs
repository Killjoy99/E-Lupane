use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Settings Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("settings heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Settings Plugin Shutting Down");
                break;
            }
        }
    }
}
