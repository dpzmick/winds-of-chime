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
