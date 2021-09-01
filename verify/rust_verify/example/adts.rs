#[macro_use] extern crate builtin_macros;
extern crate builtin;
use builtin::*;
mod pervasive;
use pervasive::*;

#[derive(Structural, PartialEq, Eq)]
struct Car<P> {
    four_doors: bool,
    passengers: P,
}

#[derive(Structural, PartialEq, Eq)]
enum Vehicle {
    Car(Car<int>),
    Train(bool),
}

fn test_struct_0(p: int) {
    let c1 = Car { four_doors: true, passengers: p };
    assert(c1.passengers == p);
    assert((Car { passengers: p, four_doors: true }).passengers == p);
}

// fn test_structural_eq(passengers: int) {
//     let c1 = Car { passengers, four_doors: true };
//     let c2 = Car { passengers, four_doors: false };
//     let c3 = Car { passengers, four_doors: true };
// 
//     assert(c1 == c3);
//     assert(c1 != c2);
// 
//     let t = Vehicle::Train(true);
//     let ca = Vehicle::Car(c1);
// 
//     assert(t != ca);
//     // assert(t == ca); // FAILS
// }

fn test_struct_1(p: int) {
    assert((Car { four_doors: true, passengers: p }).passengers == p);
    assert((Car { passengers: p, four_doors: true }).passengers == p); // fields intentionally out of order
    assert((Car { four_doors: true, passengers: p }).passengers != p); // FAILS
}

fn test_struct_2(c: Car, p: int) {
    assume(c.passengers == p);
    assert(c.passengers == p);
    assert(c.passengers != p); // FAILS
}

fn test_struct_3(p: int) {
    let c = Car { passengers: p, four_doors: true };
    assert(c.passengers == p);
    assert(!c.four_doors); // FAILS
}

fn test_struct_4(passengers: int) {
    assert((Car { passengers, four_doors: true }).passengers == passengers);
}

fn test_enum_1(passengers: int) {
    let t = Vehicle::Train(true);
    let c1 = Vehicle::Car(Car { passengers, four_doors: true });
    let c2 = Vehicle::Car(Car { passengers, four_doors: false });
}

fn test_neq(passengers: int) {
    let c1 = Car { passengers, four_doors: true };
    let c2 = Car { passengers, four_doors: false };
    let c3 = Car { passengers, four_doors: true };

    assert(c1 == c3);
    assert(c1 != c2);

    let t = Vehicle::Train(true);
    let ca = Vehicle::Car(c1);

    assert(t != ca);
    assert(t == ca); // FAILS
}

fn main() {}
