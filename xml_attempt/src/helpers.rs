use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::fmt::Display;

// take by value because there's nothinguseful to do with a None, and
// we'll panic if the optional isn't none
pub fn expect_none<T, D: Display>(o: Option<T>, msg: D) 
{
    if o.is_some() {
        panic!(format!("Expected none {}", msg))
    }
}

pub fn get_file_contents<P: AsRef<Path>>(filename: P) -> std::io::Result<String> {
    let mut f = File::open(filename)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    Ok(contents)
}
