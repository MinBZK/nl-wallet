#[derive(wallet_common_macros::ErrorCategory)]
enum Error {
    First,
    Second,
}

fn main() {
    let _ = Error::First;
}
