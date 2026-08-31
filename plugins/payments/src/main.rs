use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Payments Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("payments heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Payments Plugin Shutting Down");
                break;
            }
        }
    }
}
