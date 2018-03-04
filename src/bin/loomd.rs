extern crate loom;
use std::env::args;

pub fn main() {
    loom::daemon::run(args().collect()).unwrap().join().unwrap();
}
