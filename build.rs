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

fn main() {
    let root = path::Path::new("src").join("shaders");
    let shaders = &[
        (root.join("memcpy.glsl"), ShaderStage::Compute),
        (root.join("vert.glsl"),   ShaderStage::Vertex),
        (root.join("frag.glsl"),   ShaderStage::Fragment),
    ];

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
