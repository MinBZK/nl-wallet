use wallet_common::error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum Error {
    #[category(invalid)]
    Invalid,
}

fn main() {}
