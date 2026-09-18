fn main() {
    let lib_dir = r"C:\Program Files\AMD\AMDuProf\lib\x64\";
    println!(r"cargo::rustc-link-search=native={}", lib_dir);
    println!(r"cargo::rustc-link-lib=static=AMDProfileController");
    println!(r"cargo::rustc-link-lib=dylib=Shell32");
}