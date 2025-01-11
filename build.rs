use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    let shader_dir = PathBuf::from("shaders");
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let spirv_dir = out_dir.join("spirv");
    fs::create_dir_all(&spirv_dir).unwrap();
    println!("cargo:rerun-if-changed=shaders");
    for entry in fs::read_dir(shader_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "slang") {
            let shader_name = path.file_stem().unwrap().to_str().unwrap();
            let spirv_path = spirv_dir.join(format!("{}.spv", shader_name));
            let status = Command::new("slangc")
                .args([
                    path.to_str().unwrap(),
                    "-target", "spirv",
                    "-fvk-use-entrypoint-name",
                    "-o", spirv_path.to_str().unwrap(),
                ])
                .status()
                .expect("Failed to execute slangc");

            if !status.success() {
                panic!("Failed to compile shader: {}", shader_name);
            }
        }
    }
}
