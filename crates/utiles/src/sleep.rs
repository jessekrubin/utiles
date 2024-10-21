use tokio::time::{sleep, Duration};

pub async fn sleep0() {
    sleep(Duration::from_millis(0)).await;
}
