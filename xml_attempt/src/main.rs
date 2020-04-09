extern crate roxmltree;
extern crate clang;

#[cfg(test)]
extern crate lazy_static;

mod error;
mod helpers;
mod parser;

// ours
use parser::*;

// crates
use roxmltree as roxml;

fn print<T: std::fmt::Debug>(t: T) {
    println!("got {:?}", t);
}

fn main() {
    let filename = "vk.xml";
    let contents = helpers::get_file_contents(filename).expect("file");
    let doc = roxml::Document::parse(&contents).expect("xml");

    let parser = Parser::for_document(&doc)
        .on_platform(print)
        .on_tag(print)
        .on_basetype(print)
        .on_bitmask_definition(print)
        .on_bitmask_alias(print)
        .on_handle(print)
        .on_handle_alias(print)
        .on_enum_definition(print)
        .on_function_pointer(print)
        .parse_document()
        .expect("parser");
}
