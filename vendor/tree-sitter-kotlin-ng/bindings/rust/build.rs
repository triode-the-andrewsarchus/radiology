fn main() {
    let src_dir = std::path::Path::new("src");

    let mut c_config = cc::Build::new();
    c_config
        .std("c11")
        .include(src_dir)
        .flag_if_supported("-Wno-unused-parameter");

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    // The generated parser is unchanged upstream code, compressed for repository size.
    let compressed = src_dir.join("parser.c.gz");
    let parser_path =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("parser.c");
    let mut decoder = flate2::read::GzDecoder::new(std::fs::File::open(&compressed).unwrap());
    std::io::copy(
        &mut decoder,
        &mut std::fs::File::create(&parser_path).unwrap(),
    )
    .expect("unpack the pinned generated Kotlin parser");
    c_config.file(&parser_path);
    println!("cargo:rerun-if-changed={}", compressed.display());

    let scanner_path = src_dir.join("scanner.c");
    c_config.file(&scanner_path);
    println!("cargo:rerun-if-changed={}", scanner_path.to_str().unwrap());

    c_config.compile("tree-sitter-kotlin");
}
