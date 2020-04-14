extern crate roxmltree;

mod parser;

// ours
use parser::*;

// crates
use roxmltree as roxml;

// stdlib
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_file_contents<P: AsRef<Path>>(filename: P) -> std::io::Result<String> {
    let mut f = File::open(filename)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    Ok(contents)
}

fn print<T: std::fmt::Debug>(t: T) {
    println!("got {:#?}", t);
}

fn main() {
    let filename = "vk.xml";
    let contents = get_file_contents(filename).expect("file");
    let doc = roxml::Document::parse(&contents).expect("xml");

    Parser::for_document(&doc)
        .on_platform(print)
        .on_tag(print)
        .on_basetype(print)
        .on_bitmask_definition(print)
        .on_bitmask_alias(print)
        .on_handle(print)
        .on_handle_alias(print)
        .on_enum_definition(print)
        .on_enum_alias(print)
        .on_function_pointer(print)
        .on_struct(print)
        .on_union(print)
        .on_enum(print)
        .on_command(print)
        .on_command_alias(print)
        .on_feature(print)
        .on_extension(print)
        .parse_document();
}
