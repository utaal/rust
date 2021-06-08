extern crate builtin;
use builtin::*;
mod pervasive;
use pervasive::*;

#[derive(Eq, PartialEq)]
struct Thing<A> {
    a: A,
}

fn one(v: int) {
    let t1 = Thing { a: v };
    let t2 = Thing { a: v };
    let a: int = t2.a;
}

fn main() { }
