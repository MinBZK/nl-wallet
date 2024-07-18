use wallet_common::sentry_capture_error;

#[sentry_capture_error]
enum Foo {}

fn main() {}
