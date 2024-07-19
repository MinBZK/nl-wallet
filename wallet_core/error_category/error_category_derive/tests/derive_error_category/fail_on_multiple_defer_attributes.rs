use error_category::ErrorCategory;

#[derive(ErrorCategory)]
enum RootError {
    #[category(defer)]
    SingleStruct {
        #[defer]
        field_1: std::io::Error,
        #[defer]
        field_2: std::io::Error,
    },
}

fn main() {}
