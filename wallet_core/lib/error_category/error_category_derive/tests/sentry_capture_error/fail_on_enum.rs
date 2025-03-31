use error_category::sentry_capture_error;

#[sentry_capture_error]
enum Foo {}

fn main() {}
