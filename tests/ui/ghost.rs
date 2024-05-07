#![feature(ghost_macro)]

// let a: Ghost<nat> = 
// assert![ghost!](1 < a && a <= 3)
//
// mutating the ghost variables
//   in a for loop you need a snapshot
//   - snapshots?

// Ghost<_> as a place?

struct CopyGhost<T> {
    _p: std::marker::PhantomData<(T,)>,
}
impl<T> Clone for CopyGhost<T> {
    fn clone(&self) -> Self { todo!() }
}
impl<T> Copy for CopyGhost<T> {}
impl<T> CopyGhost<T> {
    fn new(t: T) -> Self { todo!() }
}

// struct MoveGhost<T> {
//     _p: std::marker::PhantomData<(T,)>,
// }
// 
// fn takes_ghost(a: u64, b: CopyGhost<T>) -> u64 {
//     std::marker::ghost! {
//         let c = std::marker::ghost!( b );
//         assert!(a == b);
//     };
//     22
// }

fn main() {
    // impl<T> Ghost<T>
    // consider using: fn get(self) -> T
    //
    // let v: Ghost<Snap<Vec<T>>>
    // v: Vec<T> !!! Snap<Vec<T>>

    // ghost fn snapshot<T>(&T) -> T
    let g = core::ghost::ghost! { CopyGhost::new({
        println!("👻!");
        12
    }) };
    // println!("{}", takes_ghost(43, g));
}
