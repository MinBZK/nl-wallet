use wallet_common::error_category::handle_error_category;

#[handle_error_category]
fn foo() -> u8 {
    42
}

fn main() {}
