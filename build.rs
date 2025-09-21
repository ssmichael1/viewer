fn main() {
    slint_build::compile("ui/main.slint").expect("Slint build failed");
    println!("cargo:rustc-link-lib=c++"); // libstdc++ (most Linux/GCC)

    println!("cargo:rustc-link-lib=dylib=usb-1.0");

    // Ensure the executable can find the dylib at runtime *without* DYLD_LIBRARY_PATH.
    //
    // @executable_path expands to the directory containing the final binary
    // (e.g., target/debug or target/release). We'll copy the dylib there on build.
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
}
