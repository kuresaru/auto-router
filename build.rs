fn main() {
    cc::Build::new()
        .file("src/mynfq.c")
        .compile("mynfq");
    println!("cargo:rerun-if-changed=src/mynfq.c");
    println!("cargo:rustc-link-lib=netfilter_queue");
}