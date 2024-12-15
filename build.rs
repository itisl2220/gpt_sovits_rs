fn main() {
    // 不知道为啥，我在使用LIBTORCH_USE_PYTORCH=1时使用cuda会报错必须得手动链接一下动态库，否则他会忽略cuda的动态库(libtorch 我没运行起来，环境太难搞，pytorch暂时也可以用)
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("Unable to get TARGET_OS");
    match os.as_str() {
        "linux" | "windows" => {
            if let Some(lib_path) = std::env::var_os("DEP_TCH_LIBTORCH_LIB") {
                println!(
                    "cargo:rustc-link-search=native={}",
                    lib_path.into_string().unwrap()
                );
                // println!("cargo:rustc-link-arg=-Wl,-rpath=/root/libtorch/lib",);
            }
            println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
            println!("cargo:rustc-link-arg=-Wl,--copy-dt-needed-entries");
            println!("cargo:rustc-link-arg=-ltorch");
        }
        _ => {}
    }
}
