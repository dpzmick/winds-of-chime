extern crate vkxml;
extern crate mustache;

mod names;

use std::collections::HashSet;
use std::io;

const BITFIELD_TEMPLATE: &'static str =
r#"bitflags! {
    struct {{name}} : {{base}} {
        {{#members}}
          const {{name}} = {{value}};
        {{/members}}
    }
}
"#;

struct Templates {
    bitflags_template: mustache::Template,
}

impl Templates {
    fn new() -> Self {
        Self {
            bitflags_template: mustache::compile_str(BITFIELD_TEMPLATE).unwrap(),
        }
    }
}

fn main() {
    let filename = "vk.xml";
    let contents = vkxml::get_file_contents(filename).expect("file");
    let doc = vkxml::Document::for_file_contents(&contents);

    let templates = Templates::new();
    let mut ext_tags = HashSet::new();

    println!("use bitflags::bitflags;");

    // first pass, gather the extension names
    vkxml::Parser::for_document(&doc)
        .on_tag(|t| {
            if !ext_tags.insert(t.name.to_string()) {
                panic!("duplicate tag name {}", t.name);
            }
        })
        .parse_document();

    // second pass, do some processing and generation
    vkxml::Parser::for_document(&doc)
        .on_enum(|e| {
            if e.name == "API Constants" { return; }
            if e.name == "VkPerformanceCounterDescriptionFlagBitsKHR" { return; } // FIXME
            if e.enum_type != vkxml::EnumType::BitMask { return; }
            if e.members.len() == 0 { return; } // FIXME generate something for these so that the type is valid, maybe just an alias?

            let n = names::VulkanName::new(e.name, &ext_tags);
            let map = mustache::MapBuilder::new()
                .insert_str("name", n.normalized_name.to_string())
                .insert_str("base", String::from("u32")) // FIXME what is base type?
                .insert_vec("members", |mut builder| {
                    for member in &e.members {
                        match member {
                            vkxml::EnumMember::BitPos(nm, bit) => {
                                builder = builder.push_map(|builder| {
                                    if !nm.starts_with(&n.enum_header) {
                                        panic!("name {} expected to start with {}", nm, n.enum_header);
                                    }
                                    
                                    let n = &nm[n.enum_header.len()..(nm.len()-n.bit_enum_trailer.len())];
                                    let value = 0x1 << bit;
                                    let n = if let Ok(_) = n.parse::<i64>() {
                                        format!("VALUE_{}", n)
                                    }
                                    else {
                                        n.to_string()
                                    };

                                    builder.insert_str("name", n)
                                        .insert_str("value", format!("{:#X?}", value))
                                });
                            },
                            vkxml::EnumMember::Value(nm, v) => {
                                builder = builder.push_map(|builder| {
                                    if !nm.starts_with(&n.enum_header) {
                                        panic!("name {} expected to start with {}", nm, n.enum_header);
                                    }
                                    
                                    let n = &nm[n.enum_header.len()..nm.len()];
                                    builder.insert_str("name", n)
                                        .insert_str("value", v.to_string())
                                });
                            },
                            // most aliases are just legacy, skip!
                            vkxml::EnumMember::Alias(_, _) => (), 
                        }
                    }
                    builder
                })
                .build();

            templates.bitflags_template.render_data(
                &mut io::stdout(),
                &map
            ).expect("Template expansion failed");
        })
        .parse_document();
}
