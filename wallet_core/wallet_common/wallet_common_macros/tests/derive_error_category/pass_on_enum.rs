#[derive(wallet_common_macros::ErrorCategory)]
enum Error {
    #[category(expected)]
    MyError,
}

fn main() {
    let _ = Error::MyError;
}
