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
    SingleStruct {
        field: ChildError,
    }
}

fn main() {
    let _ = RootError::SingleStruct {
        field: ChildError::Unit,
    };
}
