use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Messaging Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("messaging heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Messaging Plugin Shutting Down");
                break;
            }
        }
    }
}
