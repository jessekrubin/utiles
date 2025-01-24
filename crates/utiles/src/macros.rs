/// A macro to sleep for 0 milliseconds - useful for yielding to the event loop.
///
/// usage:
/// ```rust
/// #[tokio::main]
/// async fn main() {
///    use utiles::asleep0;
///    asleep0!();
/// }
/// ```
#[macro_export]
macro_rules! asleep0 {
    () => {
        tokio::time::sleep(tokio::time::Duration::from_millis(0)).await;
    };
}
