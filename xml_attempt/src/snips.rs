
        //     let mut base_type = None;
        //     let mut nm        = None;

        //     // two passes to deal with construction
        //     for part in member.children() {
        //         if part.node_type() == roxml::NodeType::Text {
        //             // deal with in the next loop
        //         }
        //         else if part.node_type() == roxml::NodeType::Element {
        //             // first element is always a type
        //             // second is the member name
        //             match part.tag_name().name() {
        //                 "type" => {
        //                     base_type = Some(part.children().next().unwrap().text().unwrap());
        //                 },
        //                 "name" => {
        //                     nm = Some(part.children().next().unwrap().text().unwrap());
        //                 },
        //                 "enum" => {
        //                     continue;
        //                 },
        //                 "comment" => {
        //                     continue;
        //                 },
        //                 _ => panic!("unexpected element tagged {}",
        //                             part.tag_name().name())
        //             }
        //         }
        //         else {
        //             panic!("expected element or text");
        //         }
        //     }

        //     let mut typ = Type::Base(base_type.unwrap());
        //     let mut next_style = PointerStyle::Mut;
        //     let mut in_array = false;
        //     for part in member.children() {
        //         if part.node_type() == roxml::NodeType::Text {
        //             let txt = trim(part.text().unwrap());
        //             // check if this is a full array annotation

        //             if txt.starts_with("[") && txt.ends_with("]") {
        //                 let bound = txt[1..txt.len()-1].parse::<usize>().unwrap();
        //                 typ = Type::BoundedArray(bound, Box::new(typ));
        //                 continue;
        //             }

        //             if in_array {
        //                 let bound = txt.parse::<u64>().unwrap();
        //             }
        //             else {
        //                 match txt {
        //                     "struct"        => { continue; },
        //                     ""              => { continue; },
        //                     "const"         => { next_style = PointerStyle::Const; },
        //                     "const struct"  => { next_style = PointerStyle::Const; },
        //                     "*"             => { typ = Type::Pointer(next_style, Box::new(typ)); },
        //                     // FIXME check
        //                     // example: const VkObjectTableEntryNVX* const* ppObjectTableEntries;
        //                     "* const*"      => {
        //                         typ = Type::Pointer(
        //                             PointerStyle::Mut,
        //                             Box::new(Type::Pointer(next_style, Box::new(typ))))
        //                     },
        //                     "[" => { in_array = true; },
        //                     "]" => { in_array = false; /* FIXME make array */ },
        //                     _ => panic!("unexpected text '{}'", part.text().unwrap()),
        //                 }
        //             }
        //         }
        //     }

        //     members.push(StructMember {
        //         name: nm.unwrap(),
        //         typ,
        //     });
        // }
