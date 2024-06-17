#[derive(wallet_common_macros::ErrorCategory)]
enum Error {
    #[category(defer)]
    MyError(#[defer] std::io::Error),
}

fn main() {
    let _ = Error::MyError;
}
