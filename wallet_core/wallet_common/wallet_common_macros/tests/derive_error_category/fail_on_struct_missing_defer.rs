use wallet_common::error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[category(defer)]
#[allow(dead_code)]
struct Error {
    field_1: String,
    field_2: String,
}

fn main() {}
