use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Discussions Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("discussions heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Discussions Plugin Shutting Down");
                break;
            }
        }
    }
}
