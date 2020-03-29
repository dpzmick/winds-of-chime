#[derive(Hash, PartialEq, Eq, Debug)]
enum Handle<'a> {
    Concrete(ConcreteHandle<'a>),
    Alias(AliasHandle<'a>),
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct AliasHandle<'a> {
    name: &'a str,
    aliases: &'a str,
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct ConcreteHandle<'a> {
    name:        &'a str,
    parent:      Option<&'a str>,
    is_dispatch: bool,
}

impl<'a> Handle<'a> {
    fn new(ty: roxml::Node<'a, '_>) -> Self {
        if ty.attribute("alias").is_some() {
            return Handle::Alias(AliasHandle{
                name: ty.attribute("name").unwrap(),
                aliases: ty.attribute("alias").unwrap(),
            });
        }

        let parent = ty.attribute("parent");

        let mut children = ty.children();
        let typ = children.next().unwrap();
        let text = typ.children().next().unwrap();
        let is_dispatch = match text.text() {
            Some(txt) => txt.find("NON_DISPATCH").is_none(),
            _         => panic!("not a handle definition"),
        };

        let _open_paren = children.next().unwrap();
        let _name = children.next().unwrap();
        let name = match text.text() {
            Some(txt) => txt,
            _         => panic!("not a handle definition"),
        };
        let _close_paren = children.next().unwrap();

        Handle::Concrete(ConcreteHandle {
            name,
            parent,
            is_dispatch
        })
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
enum TypeE {
    Pointer(Box<Type>),
    Base(String),
    BoundedArray(usize, Box<Type>),
    Unimplemented,
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct Type {
    mutable: bool,
    ty:      Box<TypeE>,
}

impl Type {
    fn from_ctype(ctype: &clang::Type) -> Self {
        // must be a type, FIXME check this
        match ctype.get_kind() {
            clang::TypeKind::Int =>  {
                match ctype.get_display_name().as_str() {
                    "int" => Type {
                        mutable: true,
                        ty: Box::new(TypeE::Base(String::from("i32"))),
                    },
                    "const int" => Type {
                        mutable: false,
                        ty: Box::new(TypeE::Base(String::from("i32"))),
                    },
                    _ => panic!("unhandled int type {}", ctype.get_display_name()),
                }
            },
            clang::TypeKind::Float => {
                match ctype.get_display_name().as_str() {
                    "float"  => Type {
                        mutable: true,
                        ty: Box::new(TypeE::Base(String::from("f32"))),
                    },
                    "const float"  => Type {
                        mutable: false,
                        ty: Box::new(TypeE::Base(String::from("f32"))),
                    },
                    _ => panic!("unhandled float type {}", ctype.get_display_name()),
                }
            },
            clang::TypeKind::CharS => Type {
                mutable: ctype.is_const_qualified(),
                ty: Box::new(TypeE::Base(String::from("::std::os::raw::c_char"))), // FIXME what type to use
            },
            clang::TypeKind::Record => Type {
                mutable: !ctype.is_const_qualified(),
                ty: Box::new(TypeE::Base(ctype.get_display_name())),
            },
            clang::TypeKind::Pointer => {
                let base_ctype = ctype.get_pointee_type().unwrap();
                let base = Type::from_ctype(&base_ctype);
                Type {
                    mutable: !ctype.is_const_qualified(),
                    ty: Box::new(TypeE::Pointer(Box::new(base))),
                }
            },
            clang::TypeKind::Void => Type {
                mutable: !ctype.is_const_qualified(),
                ty: Box::new(TypeE::Base(String::from("()")))
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
                    ty: Box::new(TypeE::BoundedArray(size, Box::new(base)))
                }
            },
            _ => panic!("unhandled kind {:?}", ctype.get_kind()),
        }

        // FIXME decide if these types are in c or rust terms
    }

    fn from_c_decl(decl: &str) -> (Self, String) {
        let clang = clang::Clang::new().unwrap();
        let index = clang::Index::new(&clang, false, false);

        // FIXME super slow
        // alright so this was miserably slow
        // but doing this myself is miseraby hard
        let mut tmpfile = fs::File::create("test.c").unwrap();
        write!(tmpfile, "{}", decl).unwrap();
        let tu = index.parser("test.c").parse().expect("Failed to parse");

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
                _ => panic!("should be only vardecl"),
            }
        });

        ret.unwrap()
    }
}

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
struct Types<'a> {
    typedefs: HashSet<Typedef<'a>>,   // FIXME many of these likely not useful as hashset
    bitmasks: HashSet<&'a str>,
    handles:  HashSet<Handle<'a>>,
    enums:    HashSet<&'a str>,
    structs:  HashSet<Struct<'a>>,
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
        let _seminode = children.next().unwrap();

        if children.next().is_some() {
            panic!("shouldn't have had more nodes");
        }

        alias
    }

    fn get_enum_name(enm: roxml::Node<'a, '_>) -> &'a str {
        // FIXME some of these are aliases
        enm.attribute("name").unwrap()
    }

    fn new(registry: roxml::Descendants<'a, '_>) -> Self {
        for node in registry {
            if node.has_tag_name("types") {
                let mut names = Types { // FIXME use Default trait?
                    typedefs: HashSet::new(),
                    bitmasks: HashSet::new(),
                    handles:  HashSet::new(),
                    enums:    HashSet::new(),
                    structs:  HashSet::new(),
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
                        Some("handle")   => {
                            names.handles.insert(Handle::new(ty));
                        },
                        Some("enum")     => {
                            names.enums.insert(Self::get_enum_name(ty));
                        },
                        // FIXME skipped PFN
                        Some("struct")   => {
                            names.structs.insert(Struct::new(ty));
                        }
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
