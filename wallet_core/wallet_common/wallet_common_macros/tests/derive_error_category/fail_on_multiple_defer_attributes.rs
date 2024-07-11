use wallet_common::ErrorCategory;

#[derive(ErrorCategory)]
#[allow(dead_code)]
enum RootError {
    #[category(defer)]
    SingleStruct {
        #[defer]
        field_1: std::io::Error,
        #[defer]
        field_2: std::io::Error,
    }
}

fn main() {}
