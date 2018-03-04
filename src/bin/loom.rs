extern crate loom;
use std::env::args;

pub fn main() {
    loom::client::run(args().collect());
}
