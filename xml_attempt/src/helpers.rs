use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_file_contents<P: AsRef<Path>>(filename: P) -> std::io::Result<String> {
    let mut f = File::open(filename)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    Ok(contents)
}
