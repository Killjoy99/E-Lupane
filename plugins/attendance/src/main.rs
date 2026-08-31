use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Attendance Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("attendance heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Attendance Plugin Shutting Down");
                break;
            }
        }
    }
}
