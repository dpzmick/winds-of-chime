extern crate vkxml;

mod names;

use std::collections::HashSet;

fn main() {
    let filename = "vk.xml";
    let contents = vkxml::get_file_contents(filename).expect("file");
    let doc = vkxml::Document::for_file_contents(&contents);

    let mut ext_tags = HashSet::new();

    // first pass, gather metadata
    vkxml::Parser::for_document(&doc)
        .on_tag(|t| {
            if !ext_tags.insert(t.name.to_string()) {
                panic!("duplicate tag name {}", t.name);
            }
        })
        .parse_document();

    // second pass, do some processing and generation
    vkxml::Parser::for_document(&doc)
        .on_struct(|s| {
            println!("struct: {:?}", s);
            let _ = names::VulkanName::new(s.name, &ext_tags);
        })
        .parse_document();
}
