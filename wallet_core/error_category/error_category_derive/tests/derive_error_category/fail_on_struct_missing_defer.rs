use error_category::ErrorCategory;

#[derive(ErrorCategory)]
#[category(defer)]
struct Error {
    field_1: String,
    field_2: String,
}

fn main() {}
