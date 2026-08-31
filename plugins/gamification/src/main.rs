use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Gamification Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("gamification heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Gamification Plugin Shutting Down");
                break;
            }
        }
    }
}
