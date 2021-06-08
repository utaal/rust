extern crate builtin;
extern crate builtin_macros;
use builtin::*;
use builtin_macros::*;
mod pervasive;
use pervasive::*;

#[derive(PartialEq, Eq, SmtEq)]
struct Car<X> {
    four_doors: X,
}

// fn two<T: SmtEq>(t: T) { }

fn one(v: int) {
    let c = Car { four_doors: true };
}

fn main() { }
