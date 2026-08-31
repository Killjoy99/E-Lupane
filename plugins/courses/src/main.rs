use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Courses Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("courses heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Courses Plugin Shutting Down");
                break;
            }
        }
    }
}
