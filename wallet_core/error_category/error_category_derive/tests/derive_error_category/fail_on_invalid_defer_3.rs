use error_category::ErrorCategory;

#[derive(ErrorCategory)]
enum Error {
    #[category(defer)]
    SingleStruct,
}

fn main() {}
