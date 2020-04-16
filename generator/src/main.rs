extern crate vkxml;
extern crate mustache;

mod names;

use std::collections::HashSet;
use std::io;
use std::fs;
use std::io::Write;
use std::cell::RefCell;

use serde::Serialize;

const BITFIELD_TEMPLATE: &'static str =
r#"bitflags! {
    pub struct {{name}} : {{base}} {
        {{#members}}
          const {{name}} = {{value}};
        {{/members}}
    }
}
"#;

// writer for the sys:: c repr of these structs
const STRUCT_TEMPLATE: &'static str =
r#"
#[repr(C)]
struct {{name}} {
  {{#members}}
    pub {{name}}: {{typ}},
  {{/members}}
}
"#;

#[derive(Serialize)]
struct WriterEnumMember {
    pub name:  String,
    pub value: String,
}

#[derive(Serialize)]
struct WriterEnum {
    pub name: String,
    pub base: String,
    pub members: Vec<WriterEnumMember>,
}

#[derive(Serialize)]
struct WriterStructMember {
    pub name: String,
    pub typ:  String,
}

#[derive(Serialize)]
struct WriterStruct {
    pub name: String,
    pub members: Vec<WriterStructMember>,
}

struct Writer {
    out:               io::BufWriter<fs::File>,
    bitflags_template: mustache::Template,
    struct_template:   mustache::Template,
}

impl Writer {
    fn new() -> Self {
        Self {
            out: io::BufWriter::new(fs::File::create("../src/sys.rs").expect("file")),
            bitflags_template: mustache::compile_str(BITFIELD_TEMPLATE).unwrap(),
            struct_template: mustache::compile_str(STRUCT_TEMPLATE).unwrap(),
        }
    }

    fn write_preamble(&mut self) {
        write!(self.out, "use bitflags::bitflags;").expect("write failed");
    }

    fn write_enum(&mut self, e: WriterEnum) {
        self.bitflags_template.render(&mut self.out, &e)
            .expect("Template expansion failed");
    }

    fn write_struct(&mut self, s: WriterStruct) {
        self.struct_template.render(&mut self.out, &s)
            .expect("Template expansion failed");
    }
}

fn main() {
    let filename = "vk.xml";
    let contents = vkxml::get_file_contents(filename).expect("file");
    let doc = vkxml::Document::for_file_contents(&contents);

    struct State {
        wtr: Writer,
        ext_tags: HashSet<String>,
    }

    let st = RefCell::new(State {
        wtr: Writer::new(),
        ext_tags: HashSet::new(),
    });

    let stb = || st.borrow();
    let stm = || st.borrow_mut();

    stm().wtr.write_preamble();

    // second pass, do some processing and generation
    vkxml::Parser::for_document(&doc)
        .on_tag(|t| {
            if !stm().ext_tags.insert(t.name.to_string()) {
                panic!("duplicate tag name {}", t.name);
            }
        })
        .on_struct(|s| {
            if s.name == "VkAllocationCallbacks" { return; }
            let n = names::VulkanName::new(s.name, &stb().ext_tags);
            let mut members = Vec::new();
            for m in &s.members {
                let name = match m.name {
                    "type" => "r#type",
                    _      => m.name,
                };

                let typ = match *m.typ.ty {
                    vkxml::Types::Base(n) => {
                        match n {
                            "int32_t"  => "i32".to_string(),
                            "uint32_t" => "u32".to_string(),
                            "float"    => "f32".to_string(),
                            "size_t"   => "usize".to_string(),
                            // FIXME stuck on HINSTANCE
                            // probably need to not generate the
                            // windows structs at all to get around
                            // this..
                            _ => {
                                let name =
                                    names::VulkanName::new(n, &stb().ext_tags);
                                name.normalized_name
                            },
                        }
                    },
                    _ => "u32".to_string(),
                };

                members.push(WriterStructMember {
                    name: name.to_string(),
                    typ:  typ,
                });
            }

            stm().wtr.write_struct(WriterStruct {
                name: n.normalized_name,
                members,
            });
        })
        .on_enum(|e| {
            if e.name == "API Constants" { return; }
            if e.name == "VkPerformanceCounterDescriptionFlagBitsKHR" { return; } // FIXME
            if e.enum_type != vkxml::EnumType::BitMask { return; }
            if e.members.len() == 0 { return; } // FIXME generate something for these so that the type is valid, maybe just an alias?

            // FIXME flags should be kept in the name
            let n = names::VulkanName::new(e.name, &stb().ext_tags);

            let mut members = Vec::new();
            for member in &e.members {
                match member {
                    vkxml::EnumMember::BitPos(nm, bit) => {
                        if !nm.starts_with(&n.enum_header) {
                            panic!("name {} expected to start with {}", nm, n.enum_header);
                        }

                        let n = &nm[n.enum_header.len()..(nm.len()-n.bit_enum_trailer.len())];
                        let value = 0x1 << bit;
                        let name = if let Ok(_) = n.parse::<i64>() {
                            format!("VALUE_{}", n)
                        }
                        else {
                            n.to_string()
                        };

                        members.push(WriterEnumMember {
                            name,
                            value: format!("{:#X?}", value),
                        });
                    },
                    vkxml::EnumMember::Value(nm, value) => {
                        if !nm.starts_with(&n.enum_header) {
                            panic!("name {} expected to start with {}", nm, n.enum_header);
                        }
                        
                        let name = &nm[n.enum_header.len()..nm.len()];

                        members.push(WriterEnumMember {
                            name:  name.to_string(),
                            value: value.to_string(),
                        });
                    },
                    // most aliases are just legacy, skip!
                    vkxml::EnumMember::Alias(_, _) => (), 
                }
            }

            stm().wtr.write_enum(WriterEnum {
                name: n.normalized_name,
                base: "u32".to_string(), // FIXME
                members,
            });
        })
        .parse_document();
}
