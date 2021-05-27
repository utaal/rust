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
        struct Thing { a: int }

        fn struct_ref(v: int) {
            let t = Thing { a: v };
            let a_ref = &t.a;
            assert(a_ref == &v);
        }
    } => Ok(())
}
