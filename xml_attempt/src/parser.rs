// crates
use roxmltree as roxml;

// stdlib
use std::convert::TryFrom;
use std::fs;
use std::io::Write;

/// None of the errors are recoverable.
/// We attempt to keep them human readable
pub type ParserError = String;

#[cfg(test)]
mod test {
    use super::*;

    pub fn xml_test<'a, F>(xml: &'a str, f: F)
        where F: FnOnce(roxml::Node<'_, 'a>)
    {
        let xml = roxml::Document::parse(xml).expect("bad text xml");
        println!("xml: {:?}", xml);
        f(xml.root_element())
    }
}

fn expect_attr<'a>(node: roxml::Node<'a, '_>, n: &str) -> Result<&'a str, ParserError> {
    node.attribute(n).ok_or(String::from(n))
}

// unhappy with having to allocate here
fn squash<'doc>(node: roxml::Node<'doc, '_>) -> String {
    // squash to string, skipping comments
    let mut squash = String::new();
    for child in node.children() {
        match child.node_type() {
            roxml::NodeType::Element => {
                if child.tag_name().name() == "comment" { continue; }
                for part in child.descendants() {
                    if part.node_type() != roxml::NodeType::Text { continue; }
                    squash.push_str(
                        &(String::from(" ") + part.text().unwrap())
                    );
                }
            },
            roxml::NodeType::Text => {
                squash.push_str(
                    &(String::from(" ") + child.text().unwrap())
                );
            },
            _ => panic!("unexpected node type"), // FIXME
        }

    }

    squash.push(';');
    squash
}

// FIXME errors
fn to_tu<'a, 'b>(index: &'a clang::Index, s: &'b str) -> clang::TranslationUnit<'a> {
    // FIXME temp file path
    let mut tmpfile = fs::File::create("test.c").unwrap();
    write!(tmpfile, "{}", s).unwrap();
    index.parser("test.c").parse().expect("Failed to parse")
}

#[derive(Debug)]
pub struct PlatformDefinition<'a> {
    pub name: &'a str,
    pub protect: &'a str,
    pub comment: &'a str,
}

impl<'a> TryFrom<roxml::Node<'a, '_>> for PlatformDefinition<'a> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'a, '_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: expect_attr(xml, "name")?,
            protect: expect_attr(xml, "protect")?,
            comment: expect_attr(xml, "comment")?,
        })
    }
}

#[cfg(test)]
mod test_platform {
    use super::*;

    #[test]
    fn test_platform() {
        let xml = "<platform name=\"xlib\" protect=\"VK_USE_PLATFORM_XLIB_KHR\" comment=\"X Window System, Xlib client library\"/>";
        test::xml_test(xml, |node| {
            let p = PlatformDefinition::try_from(node).expect("Should not fail");
            assert_eq!(p.name,    "xlib");
            assert_eq!(p.protect, "VK_USE_PLATFORM_XLIB_KHR");
            assert_eq!(p.comment, "X Window System, Xlib client library");
        });
    }
}

/// Vendor/Author tags
#[derive(Debug)]
pub struct TagDefinition<'a> {
    name:    &'a str,
    author:  &'a str,
    contact: &'a str,
}

impl<'a> TryFrom<roxml::Node<'a, '_>> for TagDefinition<'a> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'a, '_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: expect_attr(xml, "name")?,
            author: expect_attr(xml, "author")?,
            contact: expect_attr(xml, "contact")?,
        })
    }
}

#[cfg(test)]
mod test_tag {
    use super::*;

    #[test]
    fn test_tag() {
        let xml = "<tag name=\"ANDROID\"     author=\"Google LLC\"                    contact=\"Jesse Hall @critsec\"/>";
        test::xml_test(xml, |node| {
            let tag = TagDefinition::try_from(node).expect("Should not fail");
            assert_eq!(tag.name,    "ANDROID");
            assert_eq!(tag.author,  "Google LLC");
            assert_eq!(tag.contact, "Jesse Hall @critsec");
        })
    }
}

/// Anything that is a typedef
/// typedef struct blah blah_t;
#[derive(Debug)]
pub struct Typedef<'doc> {
    /// struct blah
    pub basetype: &'doc str,

    /// blah_t
    pub alias: &'doc str,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Typedef<'doc> {
    type Error = ParserError;

    fn try_from(xml_type: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let mut children = xml_type.children();
        match children.next() {
            Some(text) => match text.text() {
                Some("typedef ") => Ok(()), // note the space
                _ => Err(String::from("Parsing of typedef type failed. Expected text 'typedef'")),
            },
            None => Err(String::from("Missing expected text node from typedef")),
        }?;

        let base = match children.next() {
            Some(e) => match e.tag_name().name() {
                "type" => {
                    let mut children = e.children();
                    match children.next() {
                        Some(child) => {
                            match children.next() {
                                Some(_) => Err(String::from("Too many items inside of <type>")),
                                None    => child.text().ok_or(String::from("Expected text inside of <type>")),
                            }
                        },
                        None => Err(String::from("Expected children of <type>"))
                    }
                },
                _ => Err(String::from("Parsing of typedef type failed. Expected a <type> element")),
            },
            None => Err(String::from("Missing expected <type> node from typedef")),
        }?;

        match children.next() {
            Some(text) => Ok(()),
            None => Err(String::from("Missing expected text node from typedef")),
        }?;

        let alias = match children.next() {
            Some(text) => text.text()
                .ok_or(String::from("Expected text element while parsing typedef")),
            None => Err(String::from("Missing expected text node in typedef")),
        }?;

        match children.next() {
            Some(text) => match text.text() {
                Some(";") => Ok(()),
                _         => Err(String::from("Missing ';' in typedef")),
            },
            None => Err(String::from("Missing expected ';' text node in typedef")),
        }?;

        if children.next().is_some() {
            return Err(String::from("Parsing of typedef failed. Found more elements, expected none"));
        }

        Ok(Self { basetype: base, alias })
    }
}

#[cfg(test)]
mod test_typedef {
    use super::*;

    #[test]
    fn test_basetype() {
        let xml = "<type category=\"basetype\">typedef <type>uint64_t</type> <name>VkDeviceSize</name>;</type>";
        test::xml_test(xml, |node| {
            let tag = Typedef::try_from(node).expect("Should not fail");
            assert_eq!(tag.basetype, "uint64_t");
            assert_eq!(tag.alias,    "VkDeviceSize");
        })
    }

    #[test]
    fn test_bitmask() {
        let xml = "<type requires=\"VkRenderPassCreateFlagBits\" category=\"bitmask\">typedef <type>VkFlags</type> <name>VkRenderPassCreateFlags</name>;</type>";
        test::xml_test(xml, |node| {
            let tag = Typedef::try_from(node).expect("Should not fail");
            assert_eq!(tag.basetype, "VkFlags");
            assert_eq!(tag.alias,    "VkRenderPassCreateFlags");
        })
    }
}

#[derive(Debug)]
pub struct Alias<'doc> {
    pub basetype:  &'doc str,
    pub aliastype: &'doc str,
}

#[derive(Debug)]
pub struct Handle<'doc> {
    pub parent:      Option<&'doc str>,
    pub is_dispatch: bool,
    pub name:        &'doc str,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Handle<'doc> {
    type Error = ParserError;
    fn try_from(xml_type: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let mut children = xml_type.children(); // iterator
        let type_tag_txt = match children.next() {
            Some(type_tag) => {
                match type_tag.tag_name().name() {
                    "type" => {
                        let mut children = type_tag.children();
                        match children.next() {
                            Some(child) => {
                                match children.next() {
                                    Some(_) => Err(String::from("Too many items inside of <type>")),
                                    None    => child.text().ok_or(String::from("Expected text inside of <type>")),
                                }
                            },
                            None => Err(String::from("expected <type> to have child"))
                        }
                    },
                    _ => Err(String::from("Expected child to be <type>"))
                }
            },
            None => Err(String::from("Expected child"))
        }?;

        let is_non_dispatch = type_tag_txt.contains("NON_DISPATCH");

        match children.next() {
            Some(paren) => {
                match paren.text().ok_or(String::from("expected text"))? {
                    "(" => Ok(()),
                    _   => Err(String::from("expected '("))
                }
            },
            None => Err(String::from("expected more children"))
        }?;

        // <name>SomeNameHere</name>
        let name = match children.next() {
            Some(type_tag) => {
                match type_tag.tag_name().name() {
                    "name" => {
                        let mut children = type_tag.children();
                        match children.next() {
                            Some(child) => {
                                match children.next() {
                                    Some(_) => Err(String::from("Too many items inside of <name>")),
                                    None    => child.text().ok_or(String::from("Expected text inside of <name>")),
                                }
                            },
                            None => Err(String::from("expected <name> to have child"))
                        }
                    },
                    _ => Err(String::from("Expected child to be <name>"))
                }
            },
            None => Err(String::from("Expected child"))
        }?;

        match children.next() {
            Some(paren) => {
                match paren.text().ok_or(String::from("expected text"))? {
                    ")" => Ok(()),
                    _   => Err(String::from("expected '("))
                }
            },
            None => Err(String::from("expected more children"))
        }?;

        match children.next() {
            Some(_) => Err(String::from("expected no more children")),
            None    => Ok(()),
        }?;

        Ok(Self {
            parent:      xml_type.attribute("parent"),
            is_dispatch: !is_non_dispatch,
            name:        name,
        })
    }
}

#[cfg(test)]
mod test_handle {
    use super::*;

    #[test]
    fn test_handle_simple() {
        let xml = "<type category=\"handle\"><type>VK_DEFINE_HANDLE</type>(<name>VkInstance</name>)</type>";
        test::xml_test(xml, |node| {
            let handle = Handle::try_from(node).expect("Should not fail");
            assert_eq!(handle.parent,      None);
            assert_eq!(handle.is_dispatch, true);
            assert_eq!(handle.name,        "VkInstance");
        })
    }

    #[test]
    fn test_handle_advanced() {
        let xml = "<type category=\"handle\" parent=\"VkDescriptorPool\"><type>VK_DEFINE_NON_DISPATCHABLE_HANDLE</type>(<name>VkDescriptorSet</name>)</type>";
        test::xml_test(xml, |node| {
            let handle = Handle::try_from(node).expect("Should not fail");
            assert_eq!(handle.parent,      Some("VkDescriptorPool"));
            assert_eq!(handle.is_dispatch, false);
            assert_eq!(handle.name,        "VkDescriptorSet");
        })
    }
}

#[derive(Debug)]
pub struct EnumDefinition<'doc> {
    name: &'doc str,
}

#[derive(Debug)]
pub struct FunctionPointerType {
    pub return_type:    Type,
    pub argument_types: Vec<Type>,
}

#[derive(Debug)]
pub enum Types {
    FunctionPointer(FunctionPointerType),
    Pointer(Box<Type>),
    Base(String /* FIXME don't allocate? */),
    BoundedArray(usize, Box<Type>),
}

#[derive(Debug)]
pub struct Type {
    pub mutable: bool,
    pub ty:      Box<Types>,
}

impl Type {
    fn from_ctype(ctype: &clang::Type) -> Self {
        match ctype.get_kind() {
            clang::TypeKind::Int =>  {
                match ctype.get_display_name().as_str() {
                    "int" => Type {
                        mutable: true,
                        ty: Box::new(Types::Base(String::from("int"))),
                    },
                    "const int" => Type {
                        mutable: false,
                        ty: Box::new(Types::Base(String::from("int"))),
                    },
                    _ => panic!("unhandled int type {}", ctype.get_display_name()),
                }
            },
            clang::TypeKind::Float => {
                match ctype.get_display_name().as_str() {
                    "float"  => Type {
                        mutable: true,
                        ty: Box::new(Types::Base(String::from("float"))),
                    },
                    "const float"  => Type {
                        mutable: false,
                        ty: Box::new(Types::Base(String::from("float"))),
                    },
                    _ => panic!("unhandled float type {}", ctype.get_display_name()),
                }
            },
            clang::TypeKind::CharS => Type {
                mutable: ctype.is_const_qualified(),
                ty: Box::new(Types::Base(String::from("char"))), // FIXME always a char?
            },
            clang::TypeKind::Record => Type {
                mutable: !ctype.is_const_qualified(),
                ty: Box::new(Types::Base(ctype.get_display_name())),
            },
            clang::TypeKind::Pointer => {
                let base_ctype = ctype.get_pointee_type().unwrap();
                let base = Type::from_ctype(&base_ctype);
                Type {
                    mutable: !ctype.is_const_qualified(),
                    ty: Box::new(Types::Pointer(Box::new(base))),
                }
            },
            clang::TypeKind::Void => Type {
                mutable: !ctype.is_const_qualified(),
                ty: Box::new(Types::Base(String::from("void")))
            },
            // this is something like 'struct S'
            // rip the struct off and use the real name
            clang::TypeKind::Elaborated => {
                let elab = ctype.get_elaborated_type().unwrap();
                Type::from_ctype(&elab) // idk
            },
            clang::TypeKind::ConstantArray => {
                let element = ctype.get_element_type().unwrap();
                let size    = ctype.get_size().unwrap();
                let base    = Type::from_ctype(&element);
                Type {
                    mutable: !ctype.is_const_qualified(),
                    ty: Box::new(Types::BoundedArray(size, Box::new(base)))
                }
            },
            clang::TypeKind::FunctionPrototype => {
                let ret =  Type::from_ctype(&ctype.get_result_type().unwrap());
                let args = ctype.get_argument_types().unwrap().iter()
                    .map(|t| Type::from_ctype(t))
                    .collect::<Vec<_>>();

                Type {
                    mutable: !ctype.is_const_qualified(),
                    ty: Box::new(Types::FunctionPointer(FunctionPointerType {
                        return_type: ret,
                        argument_types: args,
                    }))
                }
            },
            _ => panic!("unhandled kind {:?}", ctype.get_kind()), // FIXME better error handling
        }
    }

    fn from_c_decl<'a>(clang: &'a clang::Clang, decl: &str) -> (Self, String) {
        let index = clang::Index::new(&clang, false, false);
        let tu = to_tu(&index, decl);

        let mut ret = None;
        tu.get_entity().visit_children(|child, _| {
            // for some reason, libclang things that vardecl with a
            // struct is a struct decl and a var decl
            // skip struct decl

            match child.get_kind() {
                clang::EntityKind::StructDecl => {
                    return clang::EntityVisitResult::Continue;
                },
                clang::EntityKind::VarDecl => {
                    let var_name = child.get_name().unwrap(); // this is the easiest place to get this
                    let typ      = Type::from_ctype(&child.get_type().unwrap());
                    ret = Some( (typ, var_name) );

                    return clang::EntityVisitResult::Break;
                },
                // HACK, if this is a typedef, if must be a function pointer
                // just roll with it
                clang::EntityKind::TypedefDecl => {
                    let defname = child.get_name().unwrap();
                    let typ = Type::from_ctype(&child.get_type().unwrap().get_canonical_type());
                    ret = Some( (typ, defname) );
                    return clang::EntityVisitResult::Break;
                },
                _ => panic!("should be only vardecl or structdecl, got {:?}", child.get_kind()),
            }
        });

        ret.unwrap()
    }
}

#[derive(Debug)]
pub struct FunctionPointer<'doc> {
    pub name: &'doc str,
    pub typ:  Type,
}

#[derive(Debug)]
struct Struct {
    // structs..., NOTE: some struct members have a value attribute
    //             NOTE: some struct members have noautovalidity attribute
    //             NOTE: some struct members have optional attribute
    //             NOTE: some struct members have the 'len' attribute
}

#[derive(Debug)]
pub struct EnumValueField<'a> {
    /// The vulkan name, e.g. VK_CULL_MODE_NONE
    pub name: &'a str,

    /// Whatever value was in the XML
    pub value: &'a str, // FIXME

    /// Comment, if it existed in the original document
    pub comment: Option<&'a str>,
}

#[derive(Debug)]
pub struct EnumAliasField<'a> {
    /// The vulkan name of *this field*, e.g. VK_CULL_MODE_NONE
    pub name: &'a str,

    /// The field that this field is an alias of
    pub basefield: &'a str,

    /// Comment, if it existed in the original document
    pub comment: Option<&'a str>,
}

#[derive(Debug)]
pub struct BitPosField<'a> {
    /// The vulkan name, e.g. VK_QUEUE_GRAPHICS_BIT
    pub name:    &'a str,

    /// Which bit should be set to '1' to set this field to true
    pub bitpos:  u32,

    /// Comment, if it existed in the original document
    pub comment: Option<&'a str>,
}

#[derive(Debug)]
pub enum BitMaskField<'a> {     // FIXME looks like this might be better named "enum field" since some "enums" have bitmask
    Value(EnumValueField<'a>),
    BitPos(BitPosField<'a>),
    Alias(EnumAliasField<'a>),
}

impl<'a> TryFrom<roxml::Node<'a, '_>> for BitMaskField<'a> {
    type Error = ParserError;

    fn try_from(field: roxml::Node<'a, '_>) -> Result<Self, Self::Error> {
        match field.tag_name().name() {
            "enum" => Ok(()),
            _ => Err(format!("Expected <enum> tag, found {}.", field.tag_name().name())),
        }?;

        let name = field.attribute("name")
            .ok_or(String::from("<enum> tag missing name attribute for enum {}"))?;

        // FIXME rewrite with one big match statement, less error prone probably
        let is_bit   = field.has_attribute("bitpos");
        let is_value = field.has_attribute("value");
        let is_alias = field.has_attribute("alias");

        if is_bit && !is_value && !is_alias {
            let bitpos = field.attribute("bitpos").unwrap();
            let bitpos = match bitpos.parse::<u32>() {
                Ok(bitpos) => Ok(bitpos),
                Err(_)     => Err(format!("Expecting int for bitpos, got {}", bitpos)),
            }?;

            Ok(Self::BitPos(BitPosField {
                name:    name,
                bitpos:  bitpos,
                comment: field.attribute("comment"),
            }))
        }
        else if !is_bit && is_value && !is_alias {
            let value = field.attribute("value").unwrap();
            // FIXME parse value?
            Ok(Self::Value(EnumValueField {
                name:    name,
                value:   value,
                comment: field.attribute("comment"),
            }))
        }
        else if !is_bit && !is_value && is_alias {
            let alias = field.attribute("alias").unwrap(); // FIXME check if this direction is correct
            Ok(Self::Alias(EnumAliasField {
                name:      name,
                basefield: alias,
                comment:   field.attribute("comment"),
            }))
        }
        else {
            Err(format!("<enum> tag did not have exactly one of bitpos, value, or alias for field {}", name))
        }
    }
}

#[cfg(test)]
mod test_bitmask_field {
    use super::*;

    #[test]
    fn test_bitpos() {
        let xml = "<enum bitpos=\"0\"    name=\"VK_CULL_MODE_FRONT_BIT\"/>";
        test::xml_test(xml, |node| {
            let b = BitMaskField::try_from(node).expect("should not fail");
            match b {
                BitMaskField::BitPos(bitpos) => {
                    assert_eq!(bitpos.name,    "VK_CULL_MODE_FRONT_BIT");
                    assert_eq!(bitpos.bitpos,  0);
                    assert_eq!(bitpos.comment, None);
                },
                _ => panic!("incorrect type"),
            }
        });
    }

    #[test]
    fn test_bitpos_comment() {
        let xml = "<enum bitpos=\"0\"    name=\"VK_CULL_MODE_FRONT_BIT\" comment=\"cull front\"/>";
        test::xml_test(xml, |node| {
            let b = BitMaskField::try_from(node).expect("should not fail");
            match b {
                BitMaskField::BitPos(bitpos) => {
                    assert_eq!(bitpos.name,    "VK_CULL_MODE_FRONT_BIT");
                    assert_eq!(bitpos.bitpos,  0);
                    assert_eq!(bitpos.comment, Some("cull front"));
                },
                _ => panic!("incorrect type"),
            }
        });
    }

    #[test]
    fn test_value() {
        let xml = "<enum value=\"0\"  name=\"VK_CULL_MODE_NONE\"/>";
        test::xml_test(xml, |node| {
            let b = BitMaskField::try_from(node).expect("should not fail");
            match b {
                BitMaskField::Value(value) => {
                    assert_eq!(value.name,    "VK_CULL_MODE_NONE");
                    assert_eq!(value.value,   "0");
                    assert_eq!(value.comment, None);
                },
                _ => panic!("wrong type"),
            }
        });
    }

    #[test]
    fn test_comment() {
        let xml = "<enum value=\"0\"  name=\"VK_CULL_MODE_NONE\" comment=\"no cull\"/>";
        test::xml_test(xml, |node| {
            let b = BitMaskField::try_from(node).expect("should not fail");
            match b {
                BitMaskField::Value(value) => {
                    assert_eq!(value.name,    "VK_CULL_MODE_NONE");
                    assert_eq!(value.value,   "0");
                    assert_eq!(value.comment, Some("no cull"));
                },
                _ => panic!("wrong type"),
            }
        });
    }

    // FIXME test alias
}

// Dynamically dispatch all of these callbacks so that the user
// doesn't have to specify an explict type for the callbacks that they
// are not interested in (we can't know the type statically).
pub struct Callbacks<'doc> {
    on_platform:           Option<Box<dyn FnMut(PlatformDefinition<'doc>) + 'doc>>,
    on_tag:                Option<Box<dyn FnMut(TagDefinition<'doc>) + 'doc>>,
    on_basetype:           Option<Box<dyn FnMut(Typedef<'doc>) + 'doc>>,
    on_bitmask_definition: Option<Box<dyn FnMut(Typedef<'doc>) + 'doc>>,
    on_bitmask_alias:      Option<Box<dyn FnMut(Alias<'doc>) + 'doc>>,
    on_handle:             Option<Box<dyn FnMut(Handle<'doc>) + 'doc>>,
    on_handle_alias:       Option<Box<dyn FnMut(Alias<'doc>) + 'doc>>,
    on_enum_definition:    Option<Box<dyn FnMut(EnumDefinition<'doc>) + 'doc>>,
    on_enum_alias:         Option<Box<dyn FnMut(Alias<'doc>) + 'doc>>,
    on_function_pointer:   Option<Box<dyn FnMut(FunctionPointer<'doc>) + 'doc>>,
}

impl<'doc> Callbacks<'doc> {
    fn on_plaftform(&mut self, b: PlatformDefinition<'doc>) {
        match &mut self.on_platform {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_tag(&mut self, b: TagDefinition<'doc>) {
        match &mut self.on_tag {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_basetype(&mut self, b: Typedef<'doc>) {
        match &mut self.on_basetype {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_bitmask_definition(&mut self, b: Typedef<'doc>) {
        match &mut self.on_bitmask_definition {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_bitmask_alias(&mut self, b: Alias<'doc>) {
        match &mut self.on_bitmask_alias {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_handle(&mut self, b: Handle<'doc>) {
        match &mut self.on_handle {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_handle_alias(&mut self, b: Alias<'doc>) {
        match &mut self.on_handle_alias {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_enum_definition(&mut self, b: EnumDefinition<'doc>) {
        match &mut self.on_enum_definition {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_enum_alias(&mut self, b: Alias<'doc>) {
        match &mut self.on_enum_alias {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_function_pointer(&mut self, b: FunctionPointer<'doc>) {
        match &mut self.on_function_pointer {
            Some(cb) => cb(b),
            None     => (),
        }
    }
}

pub struct Parser<'doc, 'input> {
    document:  &'doc roxml::Document<'input>,
    clang_ctx: clang::Clang,
    callbacks: Callbacks<'doc>
}

impl<'doc, 'input> Parser<'doc, 'input> {
    pub fn for_document(document: &'doc roxml::Document<'input>) -> Self {
        Self {
            document,
            clang_ctx: clang::Clang::new().expect("Failed to init clang"), // FIXME?
            callbacks: Callbacks {
                on_platform:           None,
                on_tag:                None,
                on_basetype:           None,
                on_bitmask_definition: None,
                on_bitmask_alias:      None,
                on_handle:             None,
                on_handle_alias:       None,
                on_enum_definition:    None,
                on_enum_alias:         None,
                on_function_pointer:   None,
            }
        }
    }

    pub fn on_platform<F>(mut self, f: F) -> Self
    where
        F: FnMut(PlatformDefinition<'doc>) + 'doc
    {
        self.callbacks.on_platform = Some(Box::new(f));
        self
    }

    pub fn on_tag<F>(mut self, f: F) -> Self
    where
        F: FnMut(TagDefinition<'doc>) + 'doc
    {
        self.callbacks.on_tag = Some(Box::new(f));
        self
    }

    pub fn on_basetype<F>(mut self, f: F) -> Self
    where
        F: FnMut(Typedef<'doc>) + 'doc
    {
        self.callbacks.on_basetype = Some(Box::new(f));
        self
    }

    pub fn on_bitmask_definition<F>(mut self, f: F) -> Self
    where
        F: FnMut(Typedef<'doc>) + 'doc
    {
        self.callbacks.on_bitmask_definition = Some(Box::new(f));
        self
    }

    pub fn on_bitmask_alias<F>(mut self, f: F) -> Self
    where
        F: FnMut(Alias<'doc>) + 'doc
    {
        self.callbacks.on_bitmask_alias = Some(Box::new(f));
        self
    }

    pub fn on_handle<F>(mut self, f: F) -> Self
    where
        F: FnMut(Handle<'doc>) + 'doc
    {
        self.callbacks.on_handle = Some(Box::new(f));
        self
    }

    pub fn on_handle_alias<F>(mut self, f: F) -> Self
    where
        F: FnMut(Alias<'doc>) + 'doc
    {
        self.callbacks.on_handle_alias = Some(Box::new(f));
        self
    }

    pub fn on_enum_definition<F>(mut self, f: F) -> Self
    where
        F: FnMut(EnumDefinition<'doc>) + 'doc
    {
        self.callbacks.on_enum_definition = Some(Box::new(f));
        self
    }

    pub fn on_enum_alias<F>(mut self, f: F) -> Self
    where
        F: FnMut(Alias<'doc>) + 'doc
    {
        self.callbacks.on_enum_alias = Some(Box::new(f));
        self
    }

    pub fn on_function_pointer<F>(mut self, f: F) -> Self
    where
        F: FnMut(FunctionPointer<'doc>) + 'doc
    {
        self.callbacks.on_function_pointer = Some(Box::new(f));
        self
    }

    pub fn parse_document(mut self) -> Result<(), ParserError> {
        let registry = self.document.root_element();
        for node in registry.children() {
            // NOTE: some of the nodes are Text() whitespace between elements
            match node.tag_name().name() {
                // ignore all comments
                "comment"   => continue,
                "platforms" => self.parse_platforms(node)?,
                "tags"      => self.parse_tags(node)?,
                "types"     => self.parse_types(node)?,
                "enums"     => self.parse_enums(node)?, // many of these
                _           => continue,
            }
        }

        Ok(())
    }

    fn parse_platforms(&mut self, node: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        for platform in node.children() {
            // some text nodes show up here
            if !platform.is_element() { continue; }

            let p = PlatformDefinition::try_from(platform)?;
            self.callbacks.on_plaftform(p);
        }

        Ok(())
    }

    fn parse_tags(&mut self, node: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        for tag in node.children() {
            // some text nodes show up here
            if !tag.is_element() { continue; }

            let t = TagDefinition::try_from(tag)?;
            self.callbacks.on_tag(t);
        }

        Ok(())
    }

    fn parse_types(&mut self, node: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        for xml_type in node.children() {
            // some text nodes show up here, we are skipping them
            if !xml_type.is_element() { continue; }
            let tag_name = xml_type.tag_name().name();
            if tag_name == "comment" { continue; };
            if tag_name != "type" {
                return Err(format!("Unexepected tag with name '{}' in types section", tag_name));
            }

            let category = xml_type.attribute("category");
            if category.is_none() {
                let attrs = xml_type.attributes().iter()
                    .map(|attr| attr.name())
                    .collect::<Vec<_>>();

                // I don't actually understand either of these cases
                // filter them out and move on
                let ok = (attrs.len() == 2
                          && attrs.contains(&"name")
                          && attrs.contains(&"requires"))
                    || (attrs.len() == 1
                        && attrs.contains(&"name"));

                if ok {
                    continue;
                }
                else {
                    return Err(
                        format!("Got a type node with an unexpected set of attributes. '{:?}",
                        xml_type));
                }
            }

            let category = category.unwrap();

            match category {
                "include"     => continue,
                "define"      => continue,
                "basetype"    => self.parse_basetype(xml_type)?,
                "bitmask"     => self.parse_bitmask_def(xml_type)?,
                "handle"      => self.parse_handle(xml_type)?,
                "enum"        => self.parse_enum_def(xml_type)?,
                "funcpointer" => self.parse_funcpointer(xml_type)?,
                "struct"      => self.parse_struct(xml_type)?,
                "union"       => self.parse_union(xml_type)?,

                // bail on something we don't know how to handle
                _ => return Err(format!("Got a type node with unexpected category='{}'", category)),
            }
        }

        Ok(())
    }

    fn parse_basetype(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        self.callbacks.on_basetype(Typedef::try_from(xml_type)?);
        Ok(())
    }

    fn parse_bitmask_def(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        match xml_type.attribute("alias") {
            Some(alias) => {
                match xml_type.attribute("name") {
                    Some(name) => {
                        self.callbacks.on_bitmask_alias(Alias {
                            basetype:  alias, // these names are confusing
                            aliastype: name,
                        });
                        Ok(())
                    },
                    None => Err(String::from("Expected a name attribute when alias attribute was found")),
                }
            },
            None => {
                self.callbacks.on_bitmask_definition(Typedef::try_from(xml_type)?);
                Ok(())
            }
        }
    }

    fn parse_handle(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        match xml_type.attribute("alias") {
            Some(alias) => {
                match xml_type.attribute("name") {
                    Some(name) => {
                        self.callbacks.on_handle_alias(Alias {
                            basetype:  alias,
                            aliastype: name,
                        });
                        Ok(())
                    },
                    None => Err(String::from("Expected a name attribute when alias attribute was found")),
                }
            },
            None => {
                self.callbacks.on_handle(Handle::try_from(xml_type)?);
                Ok(())
            }
        }
    }

    fn parse_enum_def(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        match xml_type.attribute("name") {
            Some(name) => {
                match xml_type.attribute("alias") {
                    Some(alias) => {
                        self.callbacks.on_enum_alias(Alias {
                            basetype:  alias,  // again, confusing. is this right?
                            aliastype: name,
                        });
                        Ok(())
                    },
                    None => {
                        self.callbacks.on_enum_definition(EnumDefinition {
                            name
                        });
                        Ok(())
                    }
                }
            },
            None => Err(String::from("Expected 'name' attribute for enum"))
        }
    }

    fn parse_funcpointer(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        let s = squash(xml_type);
        let t = Type::from_c_decl(&self.clang_ctx, &s); // FIXME errors, also this name is wrong

        // go find the name, the name we got from clang isn't right
        // FIXME is the rest of the stuff we got from clang right??
        let mut children = xml_type.children();
        match children.next() {
            Some(_) => Ok(()),
            None    => Err(String::from("Expected children")),
        }?;

        let name = match children.next() {
            Some(name_node) => match name_node.text() {
                Some(name) => Ok(name),
                None       => Err(String::from("Expected text inside of <name></name>")),
            },
            None => Err(String::from("Expected more children")),
        }?;

        self.callbacks.on_function_pointer(FunctionPointer {
            name: name,
            typ: t.0,
        });
        Ok(())
    }

    fn parse_struct(&mut self, _xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        Ok(())
    }

    fn parse_union(&mut self, _xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        Ok(())
    }

    fn parse_enums(&mut self, node: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        // assumes that the appropriate typedefs have already been seen
        let name = match node.attribute("name") {
            Some(nm) => Ok(nm),
            None => Err(String::from("<enums> tag is missing name attribute")),
        }?;

        if name == "API Constants" {
            // FIXME special case
            return Ok(());
        }

        let enum_type = match node.attribute("type") {
            Some(et) => Ok(et),
            None => Err(format!("<enums> tag for name='{}' is missing type attribute", name)),
        }?;

        match enum_type {
            "enum"    => self.parse_enum(node),
            "bitmask" => self.parse_bitmask(node, name),
            _ => Err(format!("<enums> tag had unknown type='{}'", enum_type)),
        }
    }

    fn parse_enum(&mut self, node: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        Ok(())
    }

    fn parse_bitmask(&mut self, node: roxml::Node<'doc, '_>, enum_name: &'doc str)
        -> Result<(), ParserError>
    {
        Ok(())
    }
}
