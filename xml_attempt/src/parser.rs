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
        f(xml.root_element())
    }
}

fn expect_attr<'a>(node: roxml::Node<'a, '_>, n: &str) -> Result<&'a str, ParserError> {
    node.attribute(n).ok_or(String::from(n))
}

fn get_bool_attr<'doc>(node: roxml::Node<'doc, '_>, nm: &str) -> Result<bool, ParserError> {
    if let Some(v) = node.attribute(nm) {
        match v {
            "true"  => Ok(true),
            "false" => Ok(false),
            _       => Err(format!("Expected either true of false for {}", nm)),
        }
    }
    else {
        Ok(false)
    }
}

/// get type and name for a command arg or struct/union member
fn get_type_and_name<'doc>(xml: roxml::Node<'doc, '_>)
    -> Result<(Type<'doc>, &'doc str), ParserError>
{
    // using libclang was pretty hard, doing this manually seems
    // feasible, there aren't too many cases
    let mut children = xml.children();

    let mut is_mutable = true;

    // if the first node is a text node, look for the test "const"
    // else the node should be a tag <type>
    let mut node = children.next().ok_or(String::from("Expected child when getting type/name"))?;
    if node.node_type() == roxml::NodeType::Text {
        let txt = node.text().unwrap();
        match txt {
            "const "        => is_mutable = false,
            "struct "       => (), // ignored
            "const struct " => is_mutable = false,
            _               => return Err(format!("expected 'const ' got {}", txt)),
        }

        node = children.next().ok_or(String::from("Expected child"))?;
    }

    // whatever is at `node` should be <type> now
    if node.node_type() != roxml::NodeType::Element {
        return Err(format!("Expected element, got {:?}", node.node_type()));
    }

    if node.tag_name().name() != "type" {
        return Err(format!("Expected <type>, got <{}>", node.tag_name().name()));
    }

    let type_name = {
        let mut children = node.children();
        let n = children.next().ok_or(String::from("Expected children"))?;
        n.text().ok_or(String::from("Expected text"))
    }?;

    let mut typ = Type {
        mutable: is_mutable,
        ty: Box::new(Types::Base(type_name)),
    };
    is_mutable = true;

    node = children.next().ok_or(String::from("Expected childre"))?;

    let mut pending = false;
    if node.node_type() == roxml::NodeType::Text {
        let pointer_str = node.text().ok_or(
            String::from("Expected text node in pointer section"))?;

        let pointer_str = pointer_str.as_bytes();

        let mut i = 0;
        loop {
            if i >= pointer_str.len() { break; }

            let c = pointer_str[i];
            if c == b'*' {
                if pending {
                    typ = Type {
                        mutable: is_mutable,
                        ty: Box::new(Types::Pointer(typ)),
                    };
                }

                pending = true;
                is_mutable = true;
            }
            else if c == b' ' {
                // pass
            }
            else if c == b'c' {
                if i + "onst".len() > pointer_str.len() - i { // FIXME check idx
                    return Err(
                        String::from("Expected const in pointer str, but there isn't enough string left"));
                }

                let sl = &pointer_str[(i+1)..("onst".len()+i+1)];
                if "onst".as_bytes() != sl {
                    return Err(
                        format!("Expected const in pointer str, but didn't find onst, found {:?} instead",
                                std::str::from_utf8(sl)));
                }


                // got a const, skip ahead
                i += "onst".len();
                is_mutable = false;
            }
            else {
                return Err(format!("Unexpected character in pointer section {}", c as char));
            }

            i += 1;
        }

        if pending {
            typ = Type {
                mutable: is_mutable,
                ty: Box::new(Types::Pointer(typ)),
            };
        }

        node = children.next().ok_or(String::from("Expected children"))?;
    }

    // finally, an element node with the name
    if node.node_type() != roxml::NodeType::Element {
        return Err(format!("Expected an element, got {:?}", node.node_type()));
    }

    if node.tag_name().name() != "name" {
        return Err(format!("Expected <name> element, got <{:?}>", node.tag_name().name()));
    }

    let name = {
        let mut children = node.children();
        let n = children.next().ok_or(String::from("Expected children"))?;
        n.text().ok_or(format!("Expected text, got {:?}", n.node_type()))
    }?;

    let mut maybe_node = children.next();

    // if we have another node, check if the node is an array
    if maybe_node.is_some() && maybe_node.unwrap().node_type() != roxml::NodeType::Element {
        let open = maybe_node.unwrap().text().unwrap();
        if open.len() == 1 {
            if open != "[" {
                return Err(format!("Expected [ got '{}'", open));
            }

            // could also be something like [2]

            // next is the array size
            let sz = children.next().ok_or(String::from("Expected additional children inside of []"))?;
            let sz = sz.text().ok_or(format!("Expected text"))?; // actually something like <enum>ASDAS</enum>.

            // finally, close the bracket
            let close = children.next().ok_or(String::from("Expected additional children (closing ])"))?;
            let close = close.text().ok_or(String::from("Expected text for close bracket"))?;
            if close != "]" {
                return Err(format!("Expected ] got '{}'", open));
            }

            typ = Type {
                mutable: true, // FIXME ??? pretty sure this is right
                ty: Box::new(Types::BoundedArrayStr(sz, typ))
            };
        }
        else {
            if !open.starts_with("[") || !open.ends_with("]") {
                return Err(format!("expected [...], got '{}'", open));
            }

            // the middle must be an int
            let sl = open.get(1..(open.len()-1)).unwrap();
            let sz = sl.parse::<usize>().map_err(|e| {
                format!("failed to parse int inside of [], got '{}', error={:?}", sl, e)
            })?;

            typ = Type {
                mutable: true, // FIXME ??? pretty sure this is right
                ty: Box::new(Types::BoundedArrayInt(sz, typ))
            };
        }

        maybe_node = children.next();
    }

    // only thing this could be is a comment
    if maybe_node.is_some() {
        if maybe_node.unwrap().node_type() == roxml::NodeType::Element {
            if maybe_node.unwrap().tag_name().name() != "comment" {
                return Err(
                    format!("Expected <comment>, got <{:?}>",
                            maybe_node.unwrap().tag_name().name()));
            }
        }

        maybe_node = children.next();
    }

    // shouldn't be any more
    if maybe_node.is_some() {
        return Err(String::from("Found more children when none where expected"));
    }

    return Ok(
        (typ, name)
    );
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
            Some(_) => Ok(()),
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

#[derive(Debug, PartialEq)]
pub enum Types<'doc> {
    Pointer(Type<'doc>),
    Base(&'doc str),

    /// bounded array with integer size in xml (i.e. [2])
    BoundedArrayInt(usize, Type<'doc>),

    /// bounded array with str for size in xml (i.e. [VK_SOMETHING])
    BoundedArrayStr(&'doc str, Type<'doc>),
}

#[derive(Debug, PartialEq)]
pub struct Type<'doc> {
    pub mutable: bool,
    pub ty:      Box<Types<'doc>>,
}

// not a type because we didn't need function pointer types anywhere
// else (they are always typedefed)
#[derive(Debug)]
pub struct FunctionPointer<'doc> {
    pub name:           &'doc str,
    pub return_type:    Type<'doc>,
    pub arguments:      Vec<(&'doc str, Type<'doc>)>,

    // FIXME requires
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for FunctionPointer<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        // typedef type[pointer]* (VKAPI_PTR *<name>...</name>)(
        //   <type>xxxx</type>pointers   name,
        //   ...
        // )

        let mut children = xml.children();

        // first child should be text
        let return_type = {
            let child = children.next().ok_or(
                String::from("Expected children when parsing funcptr return_type"))?;

            if child.node_type() != roxml::NodeType::Text {
                return Err(String::from("Expected return type to be Text node"));
            }

            let spl = child.text().unwrap().split_whitespace().collect::<Vec<_>>();
            if spl.len() != 4 {
                return Err(format!("Expected the lenght of split section to be 4, got {}", spl.len()));
            }

            if spl[0] != "typedef" {
                return Err(format!("Expected the string 'typedef', got '{}'", spl[0]));
            }

            if spl[2] != "(VKAPI_PTR" {
                return Err(format!("Expected the string '(VKAPI_PTR', got '{}'", spl[2]));
            }

            if spl[3] != "*" {
                return Err(format!("Expected the string '*', got '{}'", spl[3]));
            }

            let ptrspl = spl[1].splitn(2, '*').collect::<Vec<_>>();
            let ptr_cnt = if ptrspl.len() == 2 {
                for ptr in ptrspl[1].chars() {
                    if ptr != '*' {
                        return Err(format!("Strange trailing section, contains a '{}', should only be '*", ptr));
                    }
                }
                1 + ptrspl[1].len()
            }
            else {
                0
            };

            // FIXME handle const?
            // there don't seem to be any in the document
            let mut typ = Type {
                mutable: true,
                ty:      Box::new(Types::Base(ptrspl[0])),
            };

            for _ in 0..ptr_cnt {
                typ = Type {
                    mutable: true,
                    ty: Box::new(Types::Pointer(typ))
                };
            }

            typ
        };

        let name = {
            let child = children.next().ok_or(
                String::from("Expected more children (looking for <name>) while parsing funcptr"))?;

            if child.node_type() != roxml::NodeType::Element {
                return Err(format!("Expected <name> node with node_type == Element, got {:?}", child.node_type()));
            }

            if child.tag_name().name() != "name" {
                return Err(format!("Expected <name> node, got {}", child.tag_name().name()));
            }

            // the contents of this node are the name, should be exactly one elemement
            let grandchildren = child.children().collect::<Vec<_>>();
            if grandchildren.len() != 1 {
                return Err(format!("Too many children of funcpointer <name> node, got {}", grandchildren.len()));
            }

            if grandchildren[0].node_type() != roxml::NodeType::Text {
                return Err(format!("Expected <name> grandchild node with node_type == Element, got {:?}", grandchildren[0].node_type()));
            }

            grandchildren[0].text().unwrap()
        };

        // ')(' literal
        let literal = children.next().ok_or(String::from("expected Text node for )( while parsing funcptr"))?;
        let mut literal = literal.text().ok_or(String::from("Expected literal to be Text"))?.trim_end();
        if literal == ")(void);" {
            // no args
            return Ok(Self {
                name,
                return_type,
                arguments: Vec::new(),
            })
        }

        if literal != ")(" {
            return Err(format!("Expected literal ')(\n', got {}", literal));
        }

        // iterate in pairs of <type>xx</type>(whitespace or pointers)name,
        // until we hit a pair that ends in ');' instead of ','
        let mut arguments = Vec::new();

        loop {
            let typ  = children.next().ok_or("Expected <type> child while parsing funcptr arguments")?;
            let txt  = children.next().ok_or("Expected text child while parsing funcptr arguments")?;

            if typ.node_type() != roxml::NodeType::Element {
                return Err(format!("typ of argument should be Element, got {:?}", typ.node_type()))
            }

            if txt.node_type() != roxml::NodeType::Text {
                return Err(format!("txt of argument should be Text , got {:?}", txt.node_type()))
            }

            let typ = {
                let grandchildren = typ.children().collect::<Vec<_>>();
                if grandchildren.len() != 1 {
                    return Err(format!("Wrong len for funcptr arg grandchildren, got {}", grandchildren.len()));
                }

                if grandchildren[0].node_type() != roxml::NodeType::Text {
                    return Err(format!("Wrong node_type for funcptr arg grandchildren, got {:?}", grandchildren[0].node_type()));
                }

                grandchildren[0].text().unwrap()
            };

            let txt = txt.text().unwrap();
            let spl = txt.split_whitespace().collect::<Vec<_>>();

            if spl.len() != 1 && spl.len() != 2 {
                return Err(format!("txt split by whitespace had wrong len, got {}, txt was '{}'", spl.len(), txt));
            }

            let full_name = if spl.len() == 2 { spl[1] } else { spl[0] };
            let mut typ = Type {
                mutable: true,  // FIXME handle const?
                ty:      Box::new(Types::Base(typ)),
            };

            if spl.len() == 2 {
                for ptr in spl[0].chars() {
                    if ptr != '*' {
                        return Err(
                            format!("Found non-'*' char in ptr section of funcptr arg, got '{}'", ptr));
                    }

                    typ = Type {
                        mutable: true,   // FIXME handle const?
                        ty:      Box::new(Types::Pointer(typ))
                    }
                }
            }

            let name = full_name.trim_end_matches(|c| {
                c == ',' || c == ')' || c == ';'
            });

            arguments.push( (name, typ) );

            if full_name.ends_with(");") {
                break;
            }
        }

        Ok(Self {
            name,
            return_type,
            arguments,
        })
    }
}

#[cfg(test)]
mod test_function_pointer {
    use super::*;

    #[test]
    fn test_without_pointers() {
        let xml = r#"<type category="funcpointer">typedef void (VKAPI_PTR *<name>PFN_blah</name>)(
  <type>uint32_t</type>  arg1,
  <type>uint64_t</type>  arg2);</type>"#;

        test::xml_test(xml, |node| {
            let fptr = FunctionPointer::try_from(node).expect("Should not fail");
            assert_eq!(fptr.name, "PFN_blah");
            assert_eq!(fptr.return_type, Type {
                mutable: true,
                ty:      Box::new( Types::Base("void") )
            });

            assert_eq!(fptr.arguments.len(), 2);
            assert_eq!(fptr.arguments[0], ("arg1", Type {
                mutable: true,
                ty:      Box::new( Types::Base("uint32_t") )
            }));

            assert_eq!(fptr.arguments[1], ("arg2", Type {
                mutable: true,
                ty:      Box::new( Types::Base("uint64_t") )
            }));
        });
    }

    #[test]
    fn test_noarg() {
        let xml = r#"<type category="funcpointer">typedef void (VKAPI_PTR *<name>PFN_blah</name>)(void);</type>"#;
        test::xml_test(xml, |node| {
            let fptr = FunctionPointer::try_from(node).expect("Should not fail");
            assert_eq!(fptr.name, "PFN_blah");
            assert_eq!(fptr.return_type, Type {
                mutable: true,
                ty:      Box::new( Types::Base("void") )
            });

            assert_eq!(fptr.arguments.len(), 0);
        });
    }

    #[test]
    fn test_returned_pointer() {
        let xml = r#"<type category="funcpointer">typedef void* (VKAPI_PTR *<name>PFN_blah</name>)(
  <type>uint32_t</type>  arg1,
  <type>uint64_t</type>  arg2);</type>"#;

        test::xml_test(xml, |node| {
            let fptr = FunctionPointer::try_from(node).expect("Should not fail");
            assert_eq!(fptr.name, "PFN_blah");
            assert_eq!(fptr.return_type, Type {
                mutable: true,
                ty: Box::new( Types::Pointer( Type {
                    mutable: true,
                    ty: Box::new(Types::Base("void"))
                        
                }) )
            });

            assert_eq!(fptr.arguments.len(), 2);
            assert_eq!(fptr.arguments[0], ("arg1", Type {
                mutable: true,
                ty:      Box::new( Types::Base("uint32_t") )
            }));

            assert_eq!(fptr.arguments[1], ("arg2", Type {
                mutable: true,
                ty:      Box::new( Types::Base("uint64_t") )
            }));
        });
    }

    #[test]
    fn test_arg_pointer() {
        let xml = r#"<type category="funcpointer">typedef void (VKAPI_PTR *<name>PFN_blah</name>)(
  <type>uint32_t</type>* arg1,
  <type>uint64_t</type>  arg2);</type>"#;

        test::xml_test(xml, |node| {
            let fptr = FunctionPointer::try_from(node).expect("Should not fail");
            assert_eq!(fptr.name, "PFN_blah");
            assert_eq!(fptr.return_type, Type {
                mutable: true,
                ty:      Box::new( Types::Base("void") )
            });

            assert_eq!(fptr.arguments.len(), 2);
            assert_eq!(fptr.arguments[0], ("arg1", Type {
                mutable: true,
                ty: Box::new( Types::Pointer(Type {
                    mutable: true,
                    ty: Box::new( Types::Base("uint32_t") )
                        
                }) )
            }));

            assert_eq!(fptr.arguments[1], ("arg2", Type {
                mutable: true,
                ty:      Box::new( Types::Base("uint64_t") )
            }));
        });
    }

    #[test]
    fn test_const() {
        let xml = r#"<type category="funcpointer">typedef VkBool32 (VKAPI_PTR *<name>PFN_vkDebugReportCallbackEXT</name>)(
    <type>VkDebugReportFlagsEXT</type>                       flags,
    <type>VkDebugReportObjectTypeEXT</type>                  objectType,
    <type>uint64_t</type>                                    object,
    <type>size_t</type>                                      location,
    <type>int32_t</type>                                     messageCode,
    const <type>char</type>*                                 pLayerPrefix,
    const <type>char</type>*                                 pMessage,
    <type>void</type>*                                       pUserData);</type>"#;

        test::xml_test(xml, |node| {
            let fptr = FunctionPointer::try_from(node).expect("Should not fail");
            assert_eq!(fptr.name, "PFN_vkDebugReportCallbackEXT");
            assert_eq!(fptr.return_type, Type {
                mutable: true,
                ty:      Box::new( Types::Base("VkBool32") )
            });

            assert_eq!(fptr.arguments.len(), 8);
            assert_eq!(fptr.arguments[5], ("pLayerPrefix", Type {
                mutable: true,
                ty: Box::new( Types::Pointer( Type {
                    mutable: false,
                    ty: Box::new( Types::Base("char") )
                }))
            }));
        });
    }
}

/// All of these members are exacly what is in the document with very
/// little processing
#[derive(Debug)]
pub struct Member<'doc> {
    pub name:           &'doc str,
    pub typ:            Type<'doc>,
    pub values:         Option<&'doc str>,
    pub len:            Option<&'doc str>,
    pub altlen:         Option<&'doc str>,
    pub noautovalidity: bool,
    pub optional:       Option<&'doc str>,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Member<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let (typ, name) = get_type_and_name(xml)?;
        Ok(Self {
            name:           name,
            typ:            typ,
            values:         xml.attribute("values"),
            len:            xml.attribute("len"),
            altlen:         xml.attribute("altlen"),
            noautovalidity: get_bool_attr(xml, "noautovalidity")?,
            optional:       xml.attribute("optional"),
        })
    }
}

#[cfg(test)]
mod member_test {
    use super::*;

    #[test]
    fn test_simple() {
        let xml = "<member><type>uint32_t</type>        <name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type { mutable: true, ty: Box::new(Types::Base("uint32_t"))});
            assert_eq!(m.values,         None);
            assert_eq!(m.len,            None);
            assert_eq!(m.altlen,         None);
            assert_eq!(m.noautovalidity, false);
            assert_eq!(m.optional,       None);
        });
    }

    #[test]
    fn test_no_spaces() {
        let xml = "<member><type>uint32_t</type><name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type { mutable: true, ty: Box::new(Types::Base("uint32_t"))});
            assert_eq!(m.values,         None);
            assert_eq!(m.len,            None);
            assert_eq!(m.altlen,         None);
            assert_eq!(m.noautovalidity, false);
            assert_eq!(m.optional,       None);
        });
    }

    #[test]
    fn test_noautovalidity() {
        let xml = "<member noautovalidity=\"true\"><type>uint32_t</type>        <name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type { mutable: true, ty: Box::new(Types::Base("uint32_t"))});
            assert_eq!(m.values,         None);
            assert_eq!(m.len,            None);
            assert_eq!(m.altlen,         None);
            assert_eq!(m.noautovalidity, true);
            assert_eq!(m.optional,       None);
        });
    }

    #[test]
    fn test_const() {
        let xml = "<member>const <type>uint32_t</type>        <name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type { mutable: false, ty: Box::new(Types::Base("uint32_t"))});
            assert_eq!(m.values,         None);
            assert_eq!(m.len,            None);
            assert_eq!(m.altlen,         None);
            assert_eq!(m.noautovalidity, false);
            assert_eq!(m.optional,       None);
        });
    }

    #[test]
    fn test_ptr1() {
        let xml = "<member>const <type>uint32_t</type>*        <name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type {
                mutable: true,
                ty: Box::new(Types::Pointer(Type {
                    mutable: false,
                    ty: Box::new( Types::Base("uint32_t") )
                }))
            });
        });
    }

    #[test]
    fn test_ptr2() {
        let xml = "<member>const <type>uint32_t</type>**     <name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type {
                mutable: true,
                ty: Box::new(Types::Pointer(Type {
                    mutable: true,
                    ty: Box::new(Types::Pointer(Type {
                        mutable: false,
                        ty: Box::new(Types::Base("uint32_t"))
                    }))
                }))
            })
        });
    }

    #[test]
    fn test_ptr_nasty() {
        let xml = "<member>const <type>uint32_t</type>* const*     <name>width</name></member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type {
                mutable: true,
                ty: Box::new(Types::Pointer(Type {
                    mutable: false,
                    ty: Box::new(Types::Pointer(Type {
                        mutable: false,
                        ty: Box::new(Types::Base("uint32_t"))
                    }))
                }))
            })
        });
    }

    #[test]
    fn test_arr_constant() {
        let xml = "<member><type>uint32_t</type><name>width</name>[2]</member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type {
                mutable: true,
                ty: Box::new(Types::BoundedArrayInt(2, Type {
                    mutable: true,
                    ty: Box::new(Types::Base("uint32_t"))


                }))
            });
        });
    }

    #[test]
    fn test_arr_str() {
        let xml = "<member><type>uint32_t</type><name>width</name>[<enum>VK_CONSTANT_OF_SOME_SORT</enum>]</member>";
        test::xml_test(xml, |node| {
            let m = Member::try_from(node).expect("should not fail");
            assert_eq!(m.name, "width");
            assert_eq!(m.typ, Type {
                mutable: true,
                ty: Box::new(Types::BoundedArrayStr("VK_CONSTANT_OF_SOME_SORT", Type {
                    mutable: true,
                    ty: Box::new(Types::Base("uint32_t"))


                }))
            });
        });
    }

    // FIXME test rest of attributes
    // FIXME test more interesting types like function pointers?
}

#[derive(Debug)]
pub struct Struct<'doc> {
    pub name:          &'doc str,
    pub structextends: Option<&'doc str>,
    pub returnedonly:  bool,
    pub members:       Vec<Member<'doc>>,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Struct<'doc> {
    type Error = ParserError;
    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let name = xml.attribute("name").ok_or(String::from("expected name attribute on struct"))?;
        let mut members = Vec::new();
        for member in xml.children() {
            if member.node_type() == roxml::NodeType::Text { continue; }
            if member.node_type() == roxml::NodeType::Comment { continue; }
            if member.tag_name().name() == "comment" { continue; }
            members.push( Member::try_from(member)? );
        }

        Ok(Struct {
            name:          name,
            structextends: xml.attribute("structextends"),
            returnedonly:  get_bool_attr(xml, "returnedonly")?,
            members:       members,
        })
    }
}

#[cfg(test)]
mod struct_test {
    use super::*;

    // don't test members
    #[test]
    fn test_struct() {
        let xml = r#"<type category="struct" name="VkShaderStatisticsInfoAMD" returnedonly="true">
    <member><type>VkShaderStageFlags</type> <name>shaderStageMask</name></member>
    <member><type>VkShaderResourceUsageAMD</type> <name>resourceUsage</name></member>
    <member><type>uint32_t</type> <name>numPhysicalVgprs</name></member>
    <member><type>uint32_t</type> <name>numPhysicalSgprs</name></member>
    <member><type>uint32_t</type> <name>numAvailableVgprs</name></member>
    <member><type>uint32_t</type> <name>numAvailableSgprs</name></member>
    <member><type>uint32_t</type> <name>computeWorkGroupSize</name>[3]</member>
</type>
"#;
        test::xml_test(xml, |node| {
            let m = Struct::try_from(node).expect("should not fail");
            assert_eq!(m.name, "VkShaderStatisticsInfoAMD");
            assert_eq!(m.returnedonly, true);
            assert_eq!(m.structextends, None);
            assert_eq!(m.members.len(), 7);
        });
    }
}

#[derive(Debug)]
pub struct Union<'doc> {
    pub name:    &'doc str,
    pub members: Vec<Member<'doc>>,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Union<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let name = xml.attribute("name").ok_or(
            String::from("no name attribute found on union")
        )?;

        let mut members = Vec::new();
        for member in xml.children() {
            if member.node_type() == roxml::NodeType::Text { continue; }
            if member.node_type() == roxml::NodeType::Comment { continue; }
            if member.tag_name().name() == "comment" { continue; }
            members.push(Member::try_from(member)?);
        }

        Ok(Self {name, members})
    }
}

#[cfg(test)]
mod union_test {
    use super::*;

    #[test]
    fn test_union() {
        let xml = r#"<type category="union" name="VkPerformanceCounterResultKHR" comment="// Union of all the possible return types a counter result could return">
    <member><type>int32_t</type>  <name>int32</name></member>
    <member><type>int64_t</type>  <name>int64</name></member>
    <member><type>uint32_t</type> <name>uint32</name></member>
    <member><type>uint64_t</type> <name>uint64</name></member>
    <member><type>float</type>    <name>float32</name></member>
    <member><type>double</type>   <name>float64</name></member>
</type>
"#;
        test::xml_test(xml, |node| {
            let m = Union::try_from(node).expect("should not fail");
            assert_eq!(m.name, "VkPerformanceCounterResultKHR");
            assert_eq!(m.members.len(), 6);
        });
    }
}

#[derive(Debug)]
pub enum EnumMember<'doc> {
    BitPos(&'doc str, usize),
    Value(&'doc str,  &'doc str),

    /// name, basetype
    Alias(&'doc str,  &'doc str),   // FIXME use Alias type here
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for EnumMember<'doc> {
    type Error = ParserError;

    fn try_from(node: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let name = node.attribute("name").ok_or(String::from("no name found in <enum> value"))?;

        let value  = node.attribute("value");
        let bitpos = node.attribute("bitpos");
        let alias  = node.attribute("alias");

        match (value, bitpos, alias) {
            (Some(value), None, None) => {
                Ok(Self::Value(name, value))
            },
            (None, Some(bitpos), None) => {
                let bp = bitpos.parse::<usize>().map_err(|_| {
                    format!("malformed bitpos, got '{}'", bitpos)
                })?;

                Ok(Self::BitPos(name, bp))
            },
            (None, None, Some(alias)) => {
                Ok(Self::Alias(name, alias))
            },
            _ => Err(String::from("expected only one of either value and bitpos, got both"))
        }
    }
}

#[cfg(test)]
mod test_enum_value {
    use super::*;

    #[test]
    fn test_value() {
        let xml = r#"<enum value="0x123" name="VK_ASD"/>"#;
        test::xml_test(xml, |node| {
            let m = EnumMember::try_from(node).expect("should not fail");
            match m {
                EnumMember::Value(name, value) => {
                    assert_eq!(name, "VK_ASD");
                    assert_eq!(value, "0x123");
                }
                _ => panic!("wrong type"),
            }

        });
    }

    #[test]
    fn test_bitpos() {
        let xml = r#"<enum bitpos="4" name="VK_ASD"/>"#;
        test::xml_test(xml, |node| {
            let m = EnumMember::try_from(node).expect("should not fail");
            match m {
                EnumMember::BitPos(name, pos) => {
                    assert_eq!(name, "VK_ASD");
                    assert_eq!(pos, 4);
                }
                _ => panic!("wrong type"),
            }

        });
    }

    #[test]
    fn test_alias() {
        let xml = r#"<enum name="VK_ASD" alias="VK_OTHER"/>"#;
        test::xml_test(xml, |node| {
            let m = EnumMember::try_from(node).expect("should not fail");
            match m {
                EnumMember::Alias(name, alias_of) => {
                    assert_eq!(name, "VK_ASD");
                    assert_eq!(alias_of, "VK_OTHER");
                }
                _ => panic!("wrong type"),
            }

        });
    }
}

#[derive(Debug)]
pub enum EnumType {
    APIConstants, // special case
    Enum,
    BitMask,
}

#[derive(Debug)]
pub struct Enum<'doc> {
    pub name:      &'doc str,
    pub enum_type: EnumType,
    pub members:   Vec<EnumMember<'doc>>,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Enum<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let name = xml.attribute("name").ok_or(
            String::from("expected name attribute on enums tag"))?;

        let enum_type = {
            if name == "API Constants" { // special case
                EnumType::APIConstants
            }
            else {
                let typ = xml.attribute("type").ok_or(
                    String::from("Missing attribute type from enums tag"))?;

                match typ {
                    "enum" => Ok(EnumType::Enum),
                    "bitmask" => Ok(EnumType::BitMask),
                    _ => Err(format!("expected bitmask or enum, got '{}'", typ)),
                }?
            }
        };

        let mut members = Vec::new();
        for member in xml.children() {
            if member.node_type() != roxml::NodeType::Element { continue; } // some text nodes
            if member.tag_name().name() == "comment" { continue; } // some comments too
            if member.tag_name().name() == "unused" { continue; } // UGH
            members.push(EnumMember::try_from(member)?);
        }

        Ok(Enum {
            name,
            enum_type,
            members,
        })
    }
}

#[derive(Debug)]
pub struct CommandProto<'doc> {
    pub typ:  Type<'doc>,
    pub name: &'doc str,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for CommandProto<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        // we don't need all of the features of this helper, but
        // probably fine to use it here
        let (typ, name) = get_type_and_name(xml)?;
        Ok(Self { typ, name })
    }
}

#[derive(Debug)]
pub struct CommandParam<'doc> {
    pub typ:        Type<'doc>,
    pub name:       &'doc str,
    pub optional:   Option<&'doc str>,
    pub externsync: Option<&'doc str>,
    pub len:        Option<&'doc str>,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for CommandParam<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        let (typ, name) = get_type_and_name(xml)?;
        Ok(Self {
            name:           name,
            typ:            typ,
            optional:       xml.attribute("optional"),
            externsync:     xml.attribute("externsync"),
            len:            xml.attribute("len"),
        })
    }
}

// FIXME what is implicitexternsyncparam
// I'm skipping it, since it doesn't look like there's much useful I
// can do in an automated fashion with it

#[derive(Debug)]
pub struct Command<'doc> {
    pub proto:          CommandProto<'doc>,
    pub params:         Vec<CommandParam<'doc>>,
    pub successcodes:   Option<&'doc str>,
    pub errorcodes:     Option<&'doc str>,
    pub queues:         Option<&'doc str>,
    pub renderpass:     Option<&'doc str>,
    pub cmdbufferlevel: Option<&'doc str>,
    pub pipeline:       Option<&'doc str>,
}

impl<'doc> TryFrom<roxml::Node<'doc, '_>> for Command<'doc> {
    type Error = ParserError;

    fn try_from(xml: roxml::Node<'doc, '_>) -> Result<Self, Self::Error> {
        // have to fine the proto node (should be first non-text child)
        let mut children = xml.children();
        let proto_node = loop {
            let child = children.next().ok_or(
                String::from("Expected additional children while looking for proto node"))?;

            if child.node_type() != roxml::NodeType::Element { continue; }
            break child;
        };

        let proto = CommandProto::try_from(proto_node)?;

        let mut params = Vec::new();
        for param in children { // the rest
            if param.node_type() != roxml::NodeType::Element { continue; }
            if param.tag_name().name() != "param" { continue; } // skip implictexternsyncparam
            params.push(CommandParam::try_from(param)?);
        }

        Ok(Self {
            proto:          proto,
            params:         params,
            successcodes:   xml.attribute("successcodes"),
            errorcodes:     xml.attribute("errorcodes"),
            queues:         xml.attribute("queues"),
            renderpass:     xml.attribute("renderpass"),
            cmdbufferlevel: xml.attribute("cmdbufferlevel"),
            pipeline:       xml.attribute("pipeline"),
        })
    }
}

#[derive(Debug)]
pub struct EnumRequire<'doc> {
    pub name:      &'doc str,
    pub extends:   Option<&'doc str>,
    pub extnumber: Option<&'doc str>,
    pub offset:    Option<&'doc str>,
    pub bitpos:    Option<&'doc str>,
    pub dir:       Option<&'doc str>,
    pub value:     Option<&'doc str>, // some of these have escaped xml in them, undo?
}

#[derive(Debug)]
pub enum Require<'doc> {
    Type(&'doc str),
    Command(&'doc str),
}

#[derive(Debug)]
pub struct Feature<'doc> {
    // info from the feature
    pub name:   &'doc str,
    pub api:    &'doc str,
    pub number: &'doc str,
}

// #[derive(Debug)]
// pub struct Extension<'doc> {
// }

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
    on_struct:             Option<Box<dyn FnMut(Struct<'doc>) + 'doc>>,
    on_union:              Option<Box<dyn FnMut(Union<'doc>) + 'doc>>,
    on_enum:               Option<Box<dyn FnMut(Enum<'doc>) + 'doc>>,
    on_command:            Option<Box<dyn FnMut(Command<'doc>) + 'doc>>,
    on_command_alias:      Option<Box<dyn FnMut(Alias<'doc>) + 'doc>>,
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

    fn on_struct(&mut self, b: Struct<'doc>) {
        match &mut self.on_struct {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_union(&mut self, b: Union<'doc>) {
        match &mut self.on_union {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_enum(&mut self, b: Enum<'doc>) {
        match &mut self.on_enum {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_command(&mut self, b: Command<'doc>) {
        match &mut self.on_command {
            Some(cb) => cb(b),
            None     => (),
        }
    }

    fn on_command_alias(&mut self, b: Alias<'doc>) {
        match &mut self.on_command_alias {
            Some(cb) => cb(b),
            None     => (),
        }
    }
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
                on_struct:             None,
                on_union:              None,
                on_enum:               None,
                on_command:            None,
                on_command_alias:      None,
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

    pub fn on_struct<F>(mut self, f: F) -> Self
    where
        F: FnMut(Struct<'doc>) + 'doc
    {
        self.callbacks.on_struct = Some(Box::new(f));
        self
    }

    pub fn on_union<F>(mut self, f: F) -> Self
    where
        F: FnMut(Union<'doc>) + 'doc
    {
        self.callbacks.on_union = Some(Box::new(f));
        self
    }

    pub fn on_enum<F>(mut self, f: F) -> Self
    where
        F: FnMut(Enum<'doc>) + 'doc
    {
        self.callbacks.on_enum = Some(Box::new(f));
        self
    }

    pub fn on_command<F>(mut self, f: F) -> Self
    where
        F: FnMut(Command<'doc>) + 'doc
    {
        self.callbacks.on_command = Some(Box::new(f));
        self
    }

    pub fn on_command_alias<F>(mut self, f: F) -> Self
    where
        F: FnMut(Alias<'doc>) + 'doc
    {
        self.callbacks.on_command_alias = Some(Box::new(f));
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
                "commands"  => self.parse_commands(node)?,
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

    fn parse_funcpointer(&mut self, xml: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        self.callbacks.on_function_pointer(FunctionPointer::try_from(xml)?);
        Ok(())
    }

    // FIXME what is structextends
    fn parse_struct(&mut self, xml: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        self.callbacks.on_struct(Struct::try_from(xml)?);
        Ok(())
    }

    fn parse_union(&mut self, xml: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        self.callbacks.on_union(Union::try_from(xml)?);
        Ok(())
    }

    fn parse_enums(&mut self, xml: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        self.callbacks.on_enum(Enum::try_from(xml)?);
        Ok(())
    }

    fn parse_commands(&mut self, xml: roxml::Node<'doc, '_>) -> Result<(), ParserError> {
        for command in xml.children() {
            if command.node_type() != roxml::NodeType::Element { continue; }
            match command.attribute("name") {
                Some(name) => {
                    match command.attribute("alias") {
                        Some(alias) => {
                            self.callbacks.on_command_alias(Alias {
                                basetype:  alias,  // again, confusing. is this right?
                                aliastype: name,
                            });
                            Ok(())
                        },
                        None => Err(String::from("Expected 'name' attribute for command alias"))
                    }
                },
                None => {
                    self.callbacks.on_command(Command::try_from(command)?);
                    Ok(())
                }
            }?;
        }

        Ok(())
    }
}

// FIXME all of the ctors should check that the thing they were passed was actually the right tag
// FIXME test the actual parser, make sure it delivers the right callbacks (this should be done as a regression test maybe)
