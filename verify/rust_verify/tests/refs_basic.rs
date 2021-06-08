#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_with_pervasive! {
    #[test] test_basic_ref code! {
        fn basic_ref(p: int) {
            let a = p;
            let b = &a;
            assert(b == &p);
            assert(b != &p); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_with_pervasive! {
    #[test] test_struct_ref code! {
        #[derive(PartialEq, Eq, SmtEq)]
        struct Thing { a: int, b: bool }

        fn struct_ref() {
            let t = Thing { a: 12, b: true };
            let a_ref = &t.a;
            assert(a_ref == &12);
        }
    } => Ok(())
}
