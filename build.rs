extern crate bindgen;

use std::env;
use std::fs;
use std::path;
use std::io;
use std::process::Command;

#[derive(Copy, Clone)]
enum ShaderStage {
    Compute,
    Vertex,
    Fragment,
}

fn stage_str(s: ShaderStage) -> &'static str {
    match s {
        ShaderStage::Compute  => "compute",
        ShaderStage::Vertex   => "vertex",
        ShaderStage::Fragment => "fragment"
    }
}

fn compile_shaders(shaders: &[(path::PathBuf, ShaderStage)])
{
    let out_dir = env::var("OUT_DIR").unwrap();
    for (shader, stage) in shaders {
        let out = path::Path::new(&out_dir).join(shader).with_extension("spirv");

        let build = match fs::metadata(out.clone()) {
            Ok(out_md) => {
                let inp_md = fs::metadata(shader).expect(&format!("Input file {:?} failed to open", shader));
                inp_md.modified().expect("Need times") >= out_md.modified().expect("Need times")
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    true
                }
                else {
                    panic!("Output file {:?} in bad state, failed to read metadata with {:?}", out, e)
                }
            }
        };

        if !build {
            println!("Skipping {:?}", shader);
        }

        let dir = out.parent().expect("dirname failed");
        fs::create_dir_all(dir).expect("Failed to create required output dirs");

        println!("Compiling shader {:?} to {:?}", shader, out);
        let mut cmd = Command::new("glslc");
        cmd
            .arg(format!("-fshader-stage={}", stage_str(*stage)))
            .arg(shader.to_str().expect("Input path not valid UTF-8"))
            .arg("-o")
            .arg(out.to_str().expect("Output path not valid UTF-8"))
            .arg("-O");

        println!("Running '{:?}", cmd);

        let status = cmd.status()
            .expect("Failed to execute glsl compiler");

        if !status.success() {
            panic!("Build failed");
        }
    }
}

fn create_vulkan_bindings()
{
    let vk = bindgen::Builder::default()
        .header("/usr/include/vulkan/vulkan_core.h")
        .default_enum_style(bindgen::EnumVariation::NewType{ is_bitfield: false })
        .whitelist_type("Vk.*")
        .bitfield_enum("VkFlags")
        .layout_tests(false)
        .derive_copy(false)
        .derive_default(false)
        .derive_hash(false)
        .derive_debug(true)
        .generate()
        .expect("Unable to generate vulkan bindings");

    // vulkan has one giant bit mask call VkFlags
    // all flags are a member of this, but only some flags apply
    // to some situations
    // we need to somehow generate the single massive VkFlags enum

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    vk
        .write_to_file(out_path.join("vk_bindgen.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    let root = path::Path::new("src").join("shaders");
    compile_shaders(&[
        (root.join("memcpy.glsl"), ShaderStage::Compute),
        (root.join("vert.glsl"),   ShaderStage::Vertex),
        (root.join("frag.glsl"),   ShaderStage::Fragment),
    ]);

    create_vulkan_bindings();
}
