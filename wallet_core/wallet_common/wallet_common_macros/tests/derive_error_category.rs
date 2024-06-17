#[test]
fn derive_error_category() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_error_category/fail_*.rs");
    t.pass("tests/derive_error_category/pass_*.rs");
}
