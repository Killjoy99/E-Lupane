use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Learning Paths Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("learning_paths heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Learning Paths Plugin Shutting Down");
                break;
            }
        }
    }
}
