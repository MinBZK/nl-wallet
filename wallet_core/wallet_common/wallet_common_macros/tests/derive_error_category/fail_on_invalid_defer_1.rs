use wallet_common::ErrorCategory;

#[derive(ErrorCategory)]
enum Error {
    #[category(defer)]
    MyError(#[defer] std::io::Error),
}

fn main() {}
