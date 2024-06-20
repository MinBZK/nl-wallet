use wallet_common::error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum ChildError {
    #[category(expected)]
    Unit,
}

#[derive(ErrorCategory)]
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

fn main() {}
