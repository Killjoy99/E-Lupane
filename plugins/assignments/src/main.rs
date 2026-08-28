use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Assignment Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("Assignment heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Assignment Plugin Shutting Down");
                break;
            }
        }
    }
}