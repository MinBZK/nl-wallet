use wallet_common::error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum ChildError {
    #[category(expected)]
    Unit,
}

#[derive(ErrorCategory)]
#[category(defer)]
#[allow(dead_code)]
struct Error {
    msg: String,
    source: ChildError,
}

fn main() {}
