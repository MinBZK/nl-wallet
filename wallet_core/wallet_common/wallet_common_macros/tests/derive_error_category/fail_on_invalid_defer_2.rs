use wallet_common::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum Error {
    #[category(defer)]
    SingleStruct {},
}

fn main() {}
