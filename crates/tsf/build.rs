// rustc exports the #[no_mangle] COM entry points with /EXPORT, which makes link.exe warn LNK4104
// ("should be PRIVATE") for the standard DllXxx exports. The exports are correct; silence only that warning.
fn main() {
    println!("cargo:rustc-link-arg=/IGNORE:4104");
}
