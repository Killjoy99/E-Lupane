use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Certificates Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("certificates heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Certificates Plugin Shutting Down");
                break;
            }
        }
    }
}
