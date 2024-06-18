#[derive(wallet_common_macros::ErrorCategory)]
#[allow(dead_code)]
enum ChildError {
    #[category(expected)]
    Unit,
}

#[derive(wallet_common_macros::ErrorCategory)]
#[allow(dead_code)]
enum RootError {
    #[category(defer)]
    SingleStruct {},
}

fn main() {}
