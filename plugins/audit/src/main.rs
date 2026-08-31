use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Audit Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("audit heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Audit Plugin Shutting Down");
                break;
            }
        }
    }
}
