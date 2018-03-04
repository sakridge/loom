extern crate loom;
use std::env::args;

pub fn main() {
    loom::daemon::run(args().collect()).and_then(|mut x| Some(x.join().unwrap()));
}
