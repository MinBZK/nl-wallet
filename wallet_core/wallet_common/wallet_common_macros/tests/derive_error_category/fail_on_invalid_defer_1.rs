#[derive(wallet_common_macros::ErrorCategory)]
#[allow(dead_code)]
enum Error {
    #[category(defer)]
    MyError(#[defer] std::io::Error),
}

fn main() {}
