use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=./optima-ui/raylib/");
    println!("cargo:rustc-link-lib=raylib");
    println!("cargo:rustc-link-lib=msvcrt");
    println!("cargo:rustc-link-lib=OpenGL32");
    println!("cargo:rustc-link-lib=Gdi32");
    println!("cargo:rustc-link-lib=WinMM");
    // println!("cargo:rustc-link-lib=kernal32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=User32");

    let bindings = bindgen::Builder::default()
        .header("raylib/raylib.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
