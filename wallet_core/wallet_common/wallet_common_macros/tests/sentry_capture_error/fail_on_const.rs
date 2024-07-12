use wallet_common::sentry_capture_error;

#[sentry_capture_error]
const FOO: &str = "str";

fn main() {}
