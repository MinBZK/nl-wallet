#![allow(dead_code)]

use rstest::rstest;

use error_category::{Category, ErrorCategory};

#[derive(ErrorCategory)]
enum ChildError {
    #[category(expected)]
    Unit,
    #[category(unexpected)]
    Unexpected,
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

#[derive(ErrorCategory)]
#[category(expected)]
enum ErrorWithDefaultCategory {
    Expected,
    #[category(critical)]
    Critical,
}

#[derive(ErrorCategory)]
enum RootError {
    #[category(defer)]
    SingleTuple(ChildError),
    #[category(defer)]
    DoubleTuple(#[defer] ChildError, u32),
    #[category(defer)]
    SingleStruct { field: ChildError },
    #[category(defer)]
    DoubleStruct {
        field_1: u32,
        #[defer]
        field_2: ChildError,
    },
    #[category(defer)]
    Struct(EmptyStruct),
}

#[derive(ErrorCategory)]
#[category(expected)]
struct Unit;

#[derive(ErrorCategory)]
#[category(expected)]
struct EmptyTuple();

#[derive(ErrorCategory)]
#[category(critical)]
struct SingleTuple(u32);

#[derive(ErrorCategory)]
#[category(pd)]
struct DoubleTuple(u32, u32);

#[derive(ErrorCategory)]
#[category(expected)]
struct EmptyStruct {}

#[derive(ErrorCategory)]
#[category(critical)]
struct SingleStruct {
    field: u32,
}

#[derive(ErrorCategory)]
#[category(pd)]
struct DoubleStruct {
    field_1: u32,
    field_2: u32,
}

#[derive(ErrorCategory)]
#[category(defer)]
struct SingleTupleRoot(ChildError);

#[derive(ErrorCategory)]
#[category(defer)]
struct DoubleTupleRoot(#[defer] ChildError, u32);

#[derive(ErrorCategory)]
#[category(defer)]
struct SingleStructRoot {
    field: ChildError,
}

#[derive(ErrorCategory)]
#[category(defer)]
struct DoubleStructRoot {
    field_1: u32,
    #[defer]
    field_2: ChildError,
}

#[test]
fn derive_error_category() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_error_category/fail_*.rs");
}

#[rstest]
#[case(ChildError::Unit, Category::Expected)]
#[case(ChildError::Unexpected, Category::Unexpected)]
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
#[case(RootError::Struct(EmptyStruct {}), Category::Expected)]
#[case(Unit, Category::Expected)]
#[case(EmptyTuple(), Category::Expected)]
#[case(SingleTuple(42), Category::Critical)]
#[case(DoubleTuple(42, 42), Category::PersonalData)]
#[case(EmptyStruct {}, Category::Expected)]
#[case(SingleStruct { field: 42 }, Category::Critical)]
#[case(DoubleStruct { field_1: 42, field_2: 42 }, Category::PersonalData)]
#[case(SingleTupleRoot(ChildError::SingleTuple(32)), Category::Critical)]
#[case(DoubleTupleRoot(ChildError::Unit, 42), Category::Expected)]
#[case(SingleStructRoot { field: ChildError::Unit }, Category::Expected)]
#[case(DoubleStructRoot { field_1: 42, field_2: ChildError::Unit }, Category::Expected)]
#[case(ErrorWithDefaultCategory::Expected, Category::Expected)]
#[case(ErrorWithDefaultCategory::Critical, Category::Critical)]
fn derive_error_category_pass<T: ErrorCategory>(#[case] error: T, #[case] expected: Category) {
    assert_eq!(error.category(), expected);
}
