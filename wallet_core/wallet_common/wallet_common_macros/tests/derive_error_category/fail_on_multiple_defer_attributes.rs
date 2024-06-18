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
        #[defer]
        field_1: ChildError,
        #[defer]
        field_2: ChildError,
    }
}

fn main() {
    let _ = RootError::SingleStruct {
        field_1: ChildError::Unit,
        field_2: ChildError::Unit,
    };
}
