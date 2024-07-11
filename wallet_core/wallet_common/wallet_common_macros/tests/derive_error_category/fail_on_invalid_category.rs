use wallet_common::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum Error {
    #[category(invalid)]
    Invalid,
}

fn main() {}
