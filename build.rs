use std::process::Command;

fn main() {
    Command::new("make").args(&["-C", "ccode/genesis"]).status().unwrap();
    println!("cargo:rustc-link-search=native=ccode/genesis");
    println!("cargo:rustc-link-lib=static=sha256");
}
