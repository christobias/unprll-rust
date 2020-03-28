extern crate cc;

fn main() {
    cc::Build::new()
        .file("hash-to-point/crypto-ops.c")
        .compile("libhashtopoint.a");
}
