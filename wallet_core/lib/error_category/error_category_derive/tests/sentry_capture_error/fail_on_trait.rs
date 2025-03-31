use error_category::sentry_capture_error;

#[sentry_capture_error]
trait Foo {}

fn main() {}
