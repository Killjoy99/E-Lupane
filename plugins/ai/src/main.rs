use tokio::signal;

#[tokio::main]
async fn main() {
    println!("AI Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("ai heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("AI Plugin Shutting Down");
                break;
            }
        }
    }
}
