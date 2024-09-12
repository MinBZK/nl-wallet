use std::sync::OnceLock;

use tokio::runtime::{Builder, Runtime};

static ASYNC_RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn init_async_runtime() {
    _ = ASYNC_RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Could not initialize tokio runtime")
    });
}

pub fn get_async_runtime() -> &'static Runtime {
    ASYNC_RUNTIME
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

#[cfg(test)]
mod tests {
    use flutter_api_macros::async_runtime;

    async fn plus(left: i32, right: i32) -> i32 {
        left + right
    }

    #[async_runtime]
    async fn add(left: i32, right: i32) -> i32 {
        plus(left, right).await
    }

    #[test]
    fn can_invoke_async_function_in_core() {
        crate::async_runtime::init_async_runtime();
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
