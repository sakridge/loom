extern crate loom;
use std::env::args;

pub fn main() {
    loom::client::rund(args().collect());
}
