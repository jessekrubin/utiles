fn main() {
    pyo3_build_config::use_pyo3_cfgs();
    println!(
        "cargo:rustc-env=PROFILE={}",
        std::env::var("PROFILE").unwrap()
    );
}
