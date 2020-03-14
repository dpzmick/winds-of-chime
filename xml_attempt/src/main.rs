extern crate roxmltree;

use std::collections::HashSet;
use std::fs;
use std::io::Read;

use roxmltree as roxml;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Typedef<'a> {
    real_type: &'a str,
    alias:     &'a str,
}

impl<'a> Typedef<'a> {
    fn from_basetype(ty: roxml::Node<'a, '_>) -> Self {
        // these are currently all typedefs
        // just verify that, else explode

        let mut children = ty.children();
        let text = children.next().unwrap();

        match text.text() {
            Some(txt) => {
                if txt != "typedef " {
                    panic!("not a typedef");
                }
            },
            None => panic!("should have been a text node"),
        }

        // type
        let typenode = children.next().unwrap();
        let typetext = typenode.children().next().unwrap();
        let base = match typetext.text() {
            Some(txt) => txt,
            None      => panic!("should have been a text node"),
        };

        // ???
        let _ = children.next().unwrap();

        // the name
        let namenode = children.next().unwrap();
        let alias = match namenode.text() {
            Some(txt) => txt,
            None      => panic!("should have been a text node"),
        };

        // Text(;)
        let seminode = children.next().unwrap();

        if children.next().is_some() {
            panic!("shouldn't have had more nodes");
        }

        Typedef {
            real_type: &base,
            alias:     &alias,
        }
    }
}

struct Handle {
    name:        String,
    is_dispatch: bool,
}

#[derive(Debug)]
struct Types<'a> {
    typedefs: HashSet<Typedef<'a>>,

    // these are just names
    bitmasks: HashSet<&'a str>,
    handles:  HashSet<String>,
}

impl<'a> Types<'a> {
    fn get_bitmask_name(ty: roxml::Node<'a, '_>) -> &'a str {
        // these are currently all typedefs too
        // just verify that, else explode

        if ty.attribute("alias").is_some() {
            // we are just collection names
            return ty.attribute("name").unwrap();
        }

        let mut children = ty.children();
        let text = children.next().unwrap();

        match text.text() {
            Some(txt) => {
                if txt != "typedef " {
                    panic!("not a typedef");
                }
            },
            None => panic!("should have been a text node"),
        }

        // type
        let typenode = children.next().unwrap();
        let typetext = typenode.children().next().unwrap();
        let base = match typetext.text() {
            Some(txt) => txt,
            None      => panic!("should have been a text node"),
        };

        if base != "VkFlags" {
            panic!("bitmask should have been a VkFlags");
        }

        // ???
        let _ = children.next().unwrap();

        // the name
        let namenode = children.next().unwrap();
        let alias = match namenode.text() {
            Some(txt) => txt,
            None      => panic!("should have been a text node"),
        };

        // Text(;)
        let seminode = children.next().unwrap();

        if children.next().is_some() {
            panic!("shouldn't have had more nodes");
        }

        alias
    }

    fn new(registry: roxml::Descendants<'a, '_>) -> Self {
        for node in registry {
            if node.has_tag_name("types") {
                let mut names = Types {
                    typedefs: HashSet::new(),
                    bitmasks: HashSet::new(),
                    handles:  HashSet::new(),
                };

                for ty in node.children() {
                    if ty.node_type() == roxml::NodeType::Text {
                        continue;
                    }

                    if ty.node_type() == roxml::NodeType::Comment {
                        continue;
                    }

                    match ty.attribute("category") {
                        None => continue,
                        Some("include")  => continue,
                        Some("define")   => continue,   // FIXME the android defines are here..
                        Some("basetype") => {
                            names.typedefs.insert(Typedef::from_basetype(ty));
                        },
                        Some("bitmask")  => {
                            names.bitmasks.insert(Self::get_bitmask_name(ty));
                        },
                        _ => continue,
                    }
                }

                return names;
            }
        }

        panic!("didn't find node");
    }
}


#[derive(Debug)]
struct VkXml<'a> {
    extension_names: HashSet<&'a str>,
    types:           Types<'a>,

    // dispatch handles
    // nondispatch handles
    // non-opaque structs
    // unions
    // bitmasks

    // other opaque types
}

impl<'a> VkXml<'a> {
    fn get_ext_names(registry: roxml::Descendants<'a, '_>) -> HashSet<&'a str> {
        for node in registry {
            if node.has_tag_name("tags") {
                let mut ret = HashSet::new();
                for tag in node.children() {
                    if tag.node_type() == roxml::NodeType::Text {
                        continue;
                    }

                    if tag.node_type() != roxml::NodeType::Element {
                        panic!("evil unknown error");
                    }

                    ret.insert(tag.attribute("name").unwrap());
                }

                return ret;
            }
        }

        panic!("not found");
    }

    fn new(doc: &'a roxml::Document) -> Self {
        VkXml {
            extension_names: Self::get_ext_names(doc.descendants()),
            types:           Types::new(doc.descendants()),
        }
    }
}

fn get_file_contents() -> String {
    let mut f = fs::File::open("./vk.xml").expect("Failed to open xml");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("failed to read to string");

    contents
}


fn main() {
    let contents = get_file_contents();
    let doc = roxml::Document::parse(&contents).expect("Failed to parse XML");
    println!("xml: {:#?}", VkXml::new(&doc));
}
