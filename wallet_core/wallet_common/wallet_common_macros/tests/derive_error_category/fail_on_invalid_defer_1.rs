use wallet_common::error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum Error {
    #[category(defer)]
    MyError(#[defer] std::io::Error),
}

fn main() {}
