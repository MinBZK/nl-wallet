#[derive(wallet_common_macros::ErrorCategory)]
enum Error {
    MyError,
}

fn main() {
    let _ = Error::MyError;
}
