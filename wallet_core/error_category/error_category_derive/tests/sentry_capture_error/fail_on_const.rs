use error_category::sentry_capture_error;

#[sentry_capture_error]
const FOO: &str = "str";

fn main() {}
