extern crate vk_parse;
extern crate vkxml;
use std::path::Path;

use std::collections::HashSet;
use std::collections::HashMap;

mod names;

struct RegWrapper<'a> {
    notation:    Option<&'a vkxml::Notation>,
    vendor_ids:  Option<&'a vkxml::VendorIds>,
    tags:        Option<&'a vkxml::Tags>,
    definitions: Option<&'a vkxml::Definitions>,
    constants:   Option<&'a vkxml::Constants>,
    enums:       Option<&'a vkxml::Enums>,
    commands:    Option<&'a vkxml::Commands>,
    features:    Option<&'a vkxml::Features>,
    extensions:  Option<&'a vkxml::Extensions>,

    type_map:    HashMap<String, String>,
}

impl<'a> RegWrapper<'a> {
    fn new(reg: &'a vkxml::Registry) -> Self {
        let mut notation     = None;
        let mut vendor_ids   = None;
        let mut tags         = None;
        let mut definitions  = None;
        let mut constants    = None;
        let mut enums        = None;
        let mut commands     = None;
        let mut features     = None;
        let mut extensions   = None;

        for el in reg.elements.iter() {
            match el {
                vkxml::RegistryElement::Notation(_notation) => notation = Some(_notation),
                vkxml::RegistryElement::VendorIds(_vendor_ids) => vendor_ids = Some(_vendor_ids),
                vkxml::RegistryElement::Tags(_tags) => tags = Some(_tags),
                vkxml::RegistryElement::Definitions(_definitions) => definitions = Some(_definitions),
                vkxml::RegistryElement::Constants(_constants) => constants = Some(_constants),
                vkxml::RegistryElement::Enums(_enums) => enums = Some(_enums),
                vkxml::RegistryElement::Commands(_commands) => commands = Some(_commands),
                vkxml::RegistryElement::Features(_features) => features = Some(_features),
                vkxml::RegistryElement::Extensions(_extensions) => extensions = Some(_extensions),
            }
        }

        let mut type_map = HashMap::new();
        for sz in &[8, 16, 32, 64] {
            type_map.insert(format!("uint{}_t", sz), format!("u{}", sz));
            type_map.insert(format!("int{}_t", sz), format!("i{}", sz));
        }

        type_map.insert(String::from("float"), String::from("f32"));
        type_map.insert(String::from("double"), String::from("f64"));
        type_map.insert(String::from("size_t"), String::from("usize"));

        // FIXME remove
        type_map.insert(String::from("char"), String::from("::std::os::raw::c_char"));
        type_map.insert(String::from("void"), String::from("void"));

        Self {
            notation:    notation,
            vendor_ids:  vendor_ids,
            tags:        tags,
            definitions: definitions,
            constants:   constants,
            enums:       enums,
            commands:    commands,
            features:    features,
            extensions:  extensions,
            type_map:    type_map,
        }
    }

    fn find_enum(&self, name: &str) -> & vkxml::Enumeration {
        for enu in &self.enums.unwrap().elements {
            match enu {
                vkxml::EnumsElement::Enumeration(enu) => {
                    if name == enu.name { return enu }
                }
                _ => continue,
            }
        }

        panic!("enum not found {}", name);
    }

    fn get_extensions_set(&self) -> HashSet<String> {
        let mut ret = HashSet::new();
        for ext in &self.extensions.unwrap().elements {
            if ext.elements.len() == 0 { continue; }

            match &ext.author {
                Some(author) => {
                    ret.insert(author.clone());
                },
                None => {
                    if ext.name.find("KHR").is_some() {
                        ret.insert(String::from("KHR"));
                    }
                    else {
                        panic!("nohacks on ext {:?}", ext);
                    }
                }
            }
        }

        return ret;
    }

    // always drive pointer types off of field names
    fn map_struct_field_type(&self, nm: &str, hdr: Option<names::FieldHeader>) -> String {
        for el in &self.definitions.unwrap().elements {
            match el {
                //vkxml::Reference(Reference),
                vkxml::DefinitionsElement::Typedef(ty) => {
                    if ty.name == nm {
                        return names::VulkanName::new(&ty.name, &self.get_extensions_set()).normalized_name;
                    }
                },
                vkxml::DefinitionsElement::Bitmask(bs) => {
                    if bs.name == nm {
                        return names::VulkanName::new(&bs.name, &self.get_extensions_set()).normalized_name;
                    }
                },
                vkxml::DefinitionsElement::Struct(s) => {
                    if s.name == nm {
                        return names::VulkanName::new(&s.name, &self.get_extensions_set()).normalized_name;
                    }
                },
                vkxml::DefinitionsElement::Enumeration(e) => {
                    if e.name == nm {
                        return names::VulkanName::new(&e.name, &self.get_extensions_set()).normalized_name;
                    }
                }
                vkxml::DefinitionsElement::FuncPtr(fp) => {
                    if fp.name == nm {
                        // FIXME
                        return nm.to_string();
                    }
                },
                vkxml::DefinitionsElement::Handle(h) => {
                    if h.name == nm {
                        return names::VulkanName::new(&h.name, &self.get_extensions_set()).normalized_name;
                    }
                },
                vkxml::DefinitionsElement::Union(u) => {
                    if u.name == nm {
                        return names::VulkanName::new(&u.name, &self.get_extensions_set()).normalized_name;
                    }
                },
                _ => continue,
                // vkxml::DefinitionsElement::Define(Define),
            }
        }

        self.type_map[nm].clone()
    }
}

fn proc_mask(regw: &RegWrapper, mask: &vkxml::Bitmask) {
    if mask.enumref.is_none() { return; }

    let enu = regw.find_enum(mask.enumref.as_ref().unwrap());
    let nm = names::VulkanName::new(&enu.name, &regw.get_extensions_set());

    if enu.elements.len() == 0 { return; }

    println!("bitflags! {{");
    println!("    pub struct {}: {} {{", nm.normalized_name, mask.basetype);
    for ele in enu.elements.iter() {
        match ele {
            vkxml::EnumerationElement::Enum(constant) => {
                let idx = constant.name.find(&nm.enum_header);
                if idx.is_none() || idx.unwrap() != 0 {
                    panic!("failed to find header {} in {:?}", nm.enum_header, constant);
                }

                let bitidx = constant.name.find(&nm.bit_enum_trailer);
                if bitidx.is_some() && bitidx.unwrap() + nm.bit_enum_trailer.len() != constant.name.len() {
                    panic!("Failed to find trailer {} in {:?}", nm.bit_enum_trailer, constant);
                }

                let nm = if bitidx.is_some() {
                    &constant.name[nm.enum_header.len()..bitidx.unwrap()]
                }
                else {
                    &constant.name[nm.enum_header.len()..]
                };

                let value = if let Some(bitpos) = &constant.bitpos {
                    format!("1<<{}", bitpos)
                }
                else if let Some(hex) = &constant.hex {
                    format!("0x{}", hex)
                }
                else if let Some(num) = &constant.number {
                    format!("{}", num)
                }
                else {
                    panic!("not a valid bitmask enum {:?}", constant)
                };

                let all_digit = nm.chars().all(|c| c.is_ascii_digit());
                if all_digit {
                    println!("        const V_{} = {};", nm, value);
                }
                else {
                    println!("        const {} = {};", nm, value);
                }

            }
            _ => continue,
        }
    }
    println!("    }}");
    println!("}}");
}

fn proc_struct(regw: &RegWrapper, strct: &vkxml::Struct)
{
    let nm = names::VulkanName::new(&strct.name, &regw.get_extensions_set());

    if strct.elements.len() == 0 { return; } // not meaningful

    // for each struct, we want:
    // - a struct with getters and setters
    //    - some types will be the same as the struct, some will be
    //    some special wrapper
    // - a builder struct that can build the structs
    //     - probably not needed for everything? What should be excluded?

    println!("#[repr(c)]");
    println!("pub struct {} {{", nm.normalized_name);
    for el in strct.elements.iter() {
        match el {
            vkxml::StructElement::Member(fld) => {
                let fnm = names::FieldName::new(fld.name.as_ref().unwrap());
                let ty = regw.map_struct_field_type(&fld.basetype, fnm.header);

                println!("    pub {}: {},", fnm.normalized_name, "u32");
            }
            _ => continue,
        }
    }
    println!("}}")
}

fn main() {
    let registry_ir = vk_parse::parse_file(Path::new("vk.xml"));
    let reg: vkxml::Registry = registry_ir.into();
    let regw = RegWrapper::new(&reg);

    println!("use ::bitflags::bitflags;");
    println!("type VkFlags = u32;");
    for definition in &regw.definitions.unwrap().elements {
        match definition {
            vkxml::DefinitionsElement::Bitmask(mask) => proc_mask(&regw, mask),
            vkxml::DefinitionsElement::Struct(strct) => proc_struct(&regw, strct),
            _ => continue,
        }
    }
}
