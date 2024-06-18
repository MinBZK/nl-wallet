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
    SingleTuple(ChildError),
}

fn main() {
    let _ = RootError::SingleTuple(ChildError::Unit);
}
