// crates
use roxmltree as roxml;

// stdlib
use std::convert::TryFrom;

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
            let p = Platform::try_from(node).expect("Should not fail");
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
            let tag = Tag::try_from(node).expect("Should not fail");
            assert_eq!(tag.name,    "ANDROID");
            assert_eq!(tag.author,  "Google LLC");
            assert_eq!(tag.contact, "Jesse Hall @critsec");
        })
    }
}

#[derive(Debug)]
enum BitMaskDefinition<'a> {
    /// basetype from typedef
    Concrete(&'a str),
    /// other BitMask that we are aliasing
    Alias(&'a str),
}

#[derive(Debug)]
struct Handle {
}

#[derive(Debug)]
struct FunctionPointer {
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

/// <enums name="VkCullModeFlagBits" type="bitmask">
///    <enum value="0"     name="VK_CULL_MODE_NONE"/>
///    <enum bitpos="0"    name="VK_CULL_MODE_FRONT_BIT"/>
///    <enum bitpos="1"    name="VK_CULL_MODE_BACK_BIT"/>
///    <enum value="0x00000003" name="VK_CULL_MODE_FRONT_AND_BACK"/>
/// </enums>
// #[derive(Debug)]
// pub struct EnumDefinition<'a> {
//     pub name:     &'a str,
//     pub fields:   Vec<EnumsField<'a>>,
// }

/// Aliases one bitmask to another
#[derive(Debug)]
pub struct BitMaskAlias<'a> {
    pub basetype: &'a str,
    pub aliastype: &'a str,
}

// Dynamiclaly dispatch all of these callbacks so that the user
// doesn't have to specify an explict type for the callbacks that they
// are not interested in (we can't know the type statically).
// FIXME asses lifetime of closures (in the +)
pub struct Callbacks<'doc> {
    on_platform:      Option<Box<dyn FnMut(PlatformDefinition<'doc>) + 'doc>>,
    on_tag:           Option<Box<dyn FnMut(TagDefinition<'doc>) + 'doc>>,
    // on_bitmask:       Option<Box<dyn FnMut(BitMask<'doc>) + 'doc>>,
    // on_bitmask_alias: Option<Box<dyn FnMut(BitMaskAlias<'doc>) + 'doc>>,
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

    // fn on_bitmask(&mut self, b: BitMask<'doc>) {
    //     match &mut self.on_bitmask {
    //         Some(cb) => cb(b),
    //         None     => (),
    //     }
    // }

    // fn on_bitmask_alias(&mut self, b: BitMaskAlias<'doc>) {
    //     match &mut self.on_bitmask_alias {
    //         Some(cb) => cb(b),
    //         None     => (),
    //     }
    // }
}

pub struct Parser<'doc, 'input> {
    document:  &'doc roxml::Document<'input>,
    callbacks: Callbacks<'doc>
}

impl<'doc, 'input> Parser<'doc, 'input> {
    pub fn for_document(document: &'doc roxml::Document<'input>) -> Self {
        Self {
            document,
            callbacks: Callbacks {
                on_platform:      None,
                on_tag:           None,
                // on_bitmask:       None,
                // on_bitmask_alias: None,
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

    // pub fn on_bitmask<F>(mut self, f: F) -> Self
    // where
    //     F: FnMut(BitMask<'doc>) + 'doc
    // {
    //     self.callbacks.on_bitmask = Some(Box::new(f));
    //     self
    // }

    // pub fn on_bitmask_alias<F>(mut self, f: F) -> Self
    // where
    //     F: FnMut(BitMaskAlias<'doc>) + 'doc
    // {
    //     self.callbacks.on_bitmask_alias = Some(Box::new(f));
    //     self
    // }

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
                //"basetype"    => self.parse_typedef(xml_type)?,
                "basetype"    => (),
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

    fn parse_typedef_no_insert(xml_type: roxml::Node<'doc, '_>)
       -> Result<(&'doc str, &'doc str), ParserError>
    {
        let mut children = xml_type.children();
        match children.next() {
            Some(text) => match text.text() {
                Some("typedef ") => Ok(()), // note the space
                _ => Err(String::from("Parsing of typedef type failed. Expected text 'typedef'")),
            },
            None => Err(String::from("Missing expected text node from typedef")),
        }?;

        match children.next() {
            Some(e) => match e.tag_name().name() {
                "type" => Ok(()), // all good
                _ => Err(String::from("Parsing of typedef type failed. Expected a <type> element")),
            },
            None => Err(String::from("Missing expected <type> node from typedef")),
        }?;

        let base = match children.next() {
            Some(text) => text.text()
                .ok_or(String::from("Parsing of typedef type failed. Expected text")),
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
            return Err(String::from("Parsing of category=basetype type failed. Found more elements, expected none"));
        }

        Ok( (alias, base) )
    }

    // fn parse_typedef(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
    //     let (alias, base) = Self::parse_typedef_no_insert(xml_type)?;
    //     match self.typedefs.insert(alias, base) {
    //         Some(_) => Err(format!("Found multiple typedefs for alias='{}", alias)),
    //         None => Ok(()),
    //     }
    // }

    fn parse_bitmask_def(&mut self, xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        let (nm, td) = match xml_type.attribute("alias") {
            Some(alias) => {
                let nm = expect_attr(xml_type, "name")?;
                // FIXME dispatch callback here? Or should I wait
                // until I also know the concrete type
                (nm, BitMaskDefinition::Alias(alias))
            },
            None => {
                let (alias, base) = Self::parse_typedef_no_insert(xml_type)?;
                (alias, BitMaskDefinition::Concrete(base))
            }
        };

        // match self.bitmasks.insert(nm, td) {
        //     Some(_) => Err(format!("Found multiple bitmasks with name='{}", nm)),
        //     None => Ok(()),
        // }

        Ok(())
    }

    fn parse_handle(&mut self, _xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        Ok(())
    }

    fn parse_enum_def(&mut self, _xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        Ok(())
    }

    fn parse_funcpointer(&mut self, _xml_type: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
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
        // first, lookup the typedef, we are going to need it
        // FIXME some of these fail because the enum is a mix of enum/bitmask
        // those seem to go under the enums section, not the bitmask section in types
        // let def = match self.bitmasks.get(enum_name) {
        //     Some(def) => Ok(def),
        //     None      => Err(format!("No definition found for bitmask {}", enum_name)),
        // }?;

        // let mut fields = Vec::new();
        // for field in node.children() {
        //     if !field.is_element() { continue; }
        //     let f = BitMaskField::try_from(field)?;
        //     fields.push(f);
        // }

        // let bm = BitMask {
        //     name: enum_name,
        //     basetype: "VkFlags", // FIXME
        //     fields: fields,
        // };

        // self.callbacks.on_bitmask(bm);

        Ok(())
    }
}
