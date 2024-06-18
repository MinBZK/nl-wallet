use rstest::rstest;

use wallet_common::error_category::{Category, ErrorCategory};

#[derive(wallet_common_macros::ErrorCategory)]
#[allow(dead_code)]
enum ChildError {
    #[category(expected)]
    Unit,
    #[category(expected)]
    EmptyTuple(),
    #[category(critical)]
    SingleTuple(u32),
    #[category(pd)]
    DoubleTuple(u32, u32),
    #[category(expected)]
    EmptyStruct {},
    #[category(critical)]
    SingleStruct { field: u32 },
    #[category(pd)]
    DoubleStruct { field_1: u32, field_2: u32 },
}

#[derive(wallet_common_macros::ErrorCategory)]
#[allow(dead_code)]
enum RootError {
    #[category(defer)]
    SingleTuple(#[defer] ChildError),
    #[category(defer)]
    DoubleTuple(#[defer] ChildError, u32),
    #[category(defer)]
    SingleStruct {
        #[defer]
        field: ChildError,
    },
    #[category(defer)]
    DoubleStruct {
        field_1: u32,
        #[defer]
        field_2: ChildError,
    },
}

#[test]
fn derive_error_category() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_error_category/fail_*.rs");
}

#[rstest]
#[case(ChildError::Unit, Category::Expected)]
#[case(ChildError::EmptyTuple(), Category::Expected)]
#[case(ChildError::SingleTuple(42), Category::Critical)]
#[case(ChildError::DoubleTuple(42, 42), Category::PersonalData)]
#[case(ChildError::EmptyStruct {}, Category::Expected)]
#[case(ChildError::SingleStruct { field: 42 }, Category::Critical)]
#[case(ChildError::DoubleStruct { field_1: 42, field_2: 42 }, Category::PersonalData)]
#[case(RootError::SingleTuple(ChildError::Unit), Category::Expected)]
#[case(RootError::SingleTuple(ChildError::EmptyTuple()), Category::Expected)]
#[case(RootError::SingleTuple(ChildError::SingleTuple(42)), Category::Critical)]
#[case(RootError::SingleTuple(ChildError::DoubleTuple(42, 42)), Category::PersonalData)]
#[case(RootError::SingleTuple(ChildError::EmptyStruct {}), Category::Expected)]
#[case(RootError::SingleTuple(ChildError::SingleStruct { field: 42 }), Category::Critical)]
#[case(RootError::SingleTuple(ChildError::DoubleStruct { field_1: 42, field_2: 42 }), Category::PersonalData)]
#[case(RootError::DoubleTuple(ChildError::Unit, 42), Category::Expected)]
#[case(RootError::SingleStruct { field: ChildError::Unit }, Category::Expected)]
#[case(RootError::DoubleStruct { field_1: 42, field_2: ChildError::Unit }, Category::Expected)]
fn derive_error_category_pass<T: ErrorCategory>(#[case] error: T, #[case] expected: Category) {
    assert_eq!(error.category(), expected)
}
