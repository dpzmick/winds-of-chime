extern crate roxmltree;
extern crate clang;

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

    let parser = ParserBuilder::for_document(&doc)
        .on_bitmask(print)
        // .on_bitmask_alias(print)
        .parse_document()
        .expect("parser");
}
