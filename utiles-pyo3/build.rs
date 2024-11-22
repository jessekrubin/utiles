use jiff::{Unit, Zoned};

fn main() {
    pyo3_build_config::use_pyo3_cfgs();
    println!(
        "cargo:rustc-env=PROFILE={}",
        std::env::var("PROFILE").unwrap()
    );

    let build_ts = Zoned::now()
        .round(Unit::Second)
        .expect("oh no, build time error");
    // build timestamp
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_ts);
}
