use error_category::sentry_capture_error;

#[sentry_capture_error]
fn foo() -> u8 {
    42
}

fn main() {}
