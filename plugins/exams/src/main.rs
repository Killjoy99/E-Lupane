use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Exams Plugin Started");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_secs(5)
            ) => {
                println!("exams heartbeat");
            }

            _ = signal::ctrl_c() => {
                println!("Exams Plugin Shutting Down");
                break;
            }
        }
    }
}
