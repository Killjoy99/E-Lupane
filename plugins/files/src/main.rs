use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Files Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("files heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Files Plugin Shutting Down");
                break;
            }
        }
    }
}
