// Build script to ensure Cargo rebuilds when the stub template changes.

fn main() {
    println!("cargo:rerun-if-changed=../../stubs/polyglot.template");
}
