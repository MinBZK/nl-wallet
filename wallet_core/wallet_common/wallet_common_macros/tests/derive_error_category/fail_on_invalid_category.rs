use wallet_common::ErrorCategory;

#[derive(ErrorCategory)]
enum Error {
    #[category(invalid)]
    Invalid,
}

fn main() {}
