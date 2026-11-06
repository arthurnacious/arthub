// intel_tex_2's ISPC kernels include C++ objects; link the C++ runtime.
fn main() {
    println!("cargo::rustc-link-lib=stdc++");
}
