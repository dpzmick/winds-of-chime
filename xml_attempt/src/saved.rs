#[derive(Hash, PartialEq, Eq, Debug)]
struct StructMember {
    typ: Type,
    name: String,
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct Struct<'a> {
    name: &'a str,
    members: Vec<StructMember>,
}

impl<'a> Struct<'a> {
    fn new(ty: roxml::Node<'a, '_>) -> Self {
        let name = ty.attribute("name").unwrap();

        // examples of member node types
        // <member><type>int32_t</type>        <name>x</name></member>
        // <member>struct <type>VkBaseOutStructure</type>* <name>pNext</name></member>
        // <member>const struct <type>VkBaseInStructure</type>* <name>pNext</name></member>
        // <member><type>VkMemoryType</type>           <name>memoryTypes</name>[<enum>VK_MAX_MEMORY_TYPES</enum>]</member>
        // there can also be comments
        // jeez this is hard

        for member in ty.children() {
            if member.tag_name().name() != "member" { continue; }

            // attempt to convert this member into plain text so that we can parse it

            // squash to string, skipping comments
            let mut squash = String::new();
            for child in member.children() {
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
                    _ => panic!("unexpected node type"),
                }

            }

            squash.push(';');

            let member_type = Type::from_c_decl(&squash);
            println!("member type: {:?}", member_type);
        }

        Self {
            name,
            members: Vec::new(),
        }
    }
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
