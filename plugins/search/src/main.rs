use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Search Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("search heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Search Plugin Shutting Down");
                break;
            }
        }
    }
}
