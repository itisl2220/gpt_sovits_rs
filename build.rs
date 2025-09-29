fn main() {
    // // 获取 libtorch 路径
    let libtorch = std::env::var("LIBTORCH").unwrap_or_else(|_| "/libtorch".to_string());
    println!("cargo:rustc-link-search=native={}/lib", libtorch);

    // // 必须按顺序链接这些库
    println!("cargo:rustc-link-lib=dylib=torch");
    println!("cargo:rustc-link-lib=dylib=torch_cpu");
    println!("cargo:rustc-link-lib=dylib=c10");
    println!("cargo:rustc-link-lib=dylib=torch_cuda"); // 如果使用 CUDA

    // // 添加 rpath
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", libtorch);
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("Unable to get TARGET_OS");
    match os.as_str() {
        "linux" | "windows" => {
            if let Some(lib_path) = std::env::var_os("DEP_TCH_LIBTORCH_LIB") {
                println!(
                    "cargo:rustc-link-search=native={}",
                    lib_path.into_string().unwrap()
                );
            }
            println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
            // println!("cargo:rustc-link-arg=-Wl,--copy-dt-needed-entries");
            println!("cargo:rustc-link-arg=-ltorch");
        }
        _ => {}
    }
}
