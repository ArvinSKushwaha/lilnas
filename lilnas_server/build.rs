use std::{env::var, fs::DirBuilder, path::PathBuf, process::Command, str::FromStr};

fn main() {
    let build_dir = PathBuf::from_str(&var("OUT_DIR").unwrap())
        .unwrap()
        .join("go_build");
    DirBuilder::new()
        .recursive(true)
        .create(build_dir.to_str().unwrap())
        .unwrap();
    println!(
        "{}",
        String::from_utf8_lossy(
            &Command::new("go")
                .current_dir("go")
                .args([
                    "build",
                    "-a",
                    "-v",
                    "-buildmode=c-archive",
                    "-o",
                    build_dir.join("libnas.a").to_str().unwrap(),
                ])
                .output()
                .unwrap()
                .stderr
        )
    );
    let lib = "nas";
    let mut path: PathBuf = build_dir.parent().unwrap().to_path_buf();
    path.push("go_build");

    println!("cargo:rerun-if-changed=go");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-flags=-l framework=CoreFoundation -l framework=Security");
    }
    println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static={}", lib);
}
