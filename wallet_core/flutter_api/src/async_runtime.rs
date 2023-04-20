use once_cell::sync::OnceCell;
use tokio::runtime::{Builder, Runtime};

static ASYNC_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn try_init_async() -> Result<(), Runtime> {
    ASYNC_RUNTIME.set(
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio Runtime."),
    )
}

#[allow(dead_code)] // At the moment only used by unit tests by `async_runtime` macro
pub fn get_async_runtime() -> &'static Runtime {
    ASYNC_RUNTIME
        .get()
        .expect("CORE must be initialized first. Please execute `init_async` first.")
}

#[cfg(test)]
mod tests {
    async fn plus(left: i32, right: i32) -> i32 {
        left + right
    }

    #[flutter_api_macros::async_runtime]
    async fn add(left: i32, right: i32) -> i32 {
        plus(left, right).await
    }

    #[test]
    fn can_invoke_async_function_in_core() {
        let _ = crate::async_runtime::try_init_async();
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
