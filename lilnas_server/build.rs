use std::{env::var, fs::DirBuilder, path::PathBuf, process::Command, str::FromStr};

fn main() {
    DirBuilder::new()
        .recursive(true)
        .create("go_build")
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
                    "../go_build/libnas.a",
                ])
                .output()
                .unwrap()
                .stderr
        )
    );
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let lib = "nas";
    let mut path: PathBuf = PathBuf::from_str(&manifest_dir).unwrap();
    path.push("go_build");

    println!("cargo:rerun-if-changed=go");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-flags=-l framework=CoreFoundation -l framework=Security");
    }
    println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static={}", lib);
}
