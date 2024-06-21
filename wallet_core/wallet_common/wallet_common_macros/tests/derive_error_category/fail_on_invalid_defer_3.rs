use wallet_common::error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum Error {
    #[category(defer)]
    SingleStruct,
}

fn main() {}
