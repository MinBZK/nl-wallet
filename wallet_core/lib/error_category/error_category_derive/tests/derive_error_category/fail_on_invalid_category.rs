use error_category::ErrorCategory;

#[derive(ErrorCategory)]
enum Error {
    #[category(invalid)]
    Invalid,
}

fn main() {}
