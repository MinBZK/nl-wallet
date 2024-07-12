use wallet_common::ErrorCategory;

#[derive(ErrorCategory)]
enum Error {
    #[category(defer)]
    SingleStruct,
}

fn main() {}
