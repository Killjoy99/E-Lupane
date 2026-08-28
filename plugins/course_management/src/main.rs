#[tokio::main]
async fn main() {
    println!("Course Management Plugin Started");

    loop {
        tokio::time::sleep(
            tokio::time::Duration::from_secs(5)
        )
        .await;

        println!("Course Management heartbeat");
    }
}