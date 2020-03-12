use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum FieldHeader {
    S,  // sType is the only thing that has this I think
    P,  // pointer
    PP, // pointer pointer
}

// can extract a lot of info from the field name, then ignore
// the rest of the info on the vkxml struct
pub struct FieldName {
    pub header:          Option<FieldHeader>,
    pub normalized_name: String,
}

impl FieldName {
    pub fn new(vkname: &str) -> Self {
        enum Mode {
            MaybeS,
            MaybeS2,
            MaybeP,
            MaybePP,
            Word,
            UpperWord,
        }

        let mut mode = Mode::MaybeS;
        let mut partial = String::new();
        let mut header = None;
        let mut parts = Vec::new();
        let mut last_was_upper = false;
        for c in vkname.chars() {
            partial.push(c);
            mode = match mode {
                Mode::MaybeS => {
                    if c == 's' {
                        Mode::MaybeS2
                    }
                    else if c == 'p' {
                        Mode::MaybeP
                    }
                    else {
                        Mode::Word
                    }
                },
                Mode::MaybeS2 => {
                    if c.is_uppercase() { // starting new word
                        header = Some(FieldHeader::S);
                        partial = partial[1..].to_string();
                        Mode::Word
                    }
                    else { // the word starts with 's'
                        Mode::Word
                    }
                }
                Mode::MaybeP => {
                    if c == 'p' {
                        Mode::MaybePP
                    }
                    else if c.is_uppercase() {
                        header = Some(FieldHeader::P);
                        partial = partial[1..].to_string();
                        Mode::Word
                    }
                    else {
                        Mode::Word
                    }
                },
                Mode::MaybePP => {
                    // next char better start a new word
                    if !c.is_uppercase() {
                        panic!("expected start of new word");
                    }

                    header = Some(FieldHeader::PP);
                    partial = partial[2..].to_string();
                    Mode::Word
                },
                Mode::Word => {
                    if c.is_uppercase() && !last_was_upper {
                        if partial.len() > 1 {
                            let start = partial.pop().unwrap();
                            parts.push(partial);
                            partial = String::new();
                            partial.push(start);
                        }
                        Mode::Word
                    }
                    else if c.is_uppercase() && last_was_upper {
                        Mode::UpperWord
                    }
                    else {
                        Mode::Word
                    }
                },
                Mode::UpperWord => {
                    if c.is_lowercase() {
                        let this_char = partial.pop().unwrap();
                        let start_char = partial.pop().unwrap();
                        parts.push(partial);
                        partial = String::new();
                        partial.push(start_char);
                        partial.push(this_char);
                        Mode::Word
                    }
                    else {
                        Mode::UpperWord
                    }
                }
            };

            last_was_upper = c.is_uppercase();
        }

        if partial.len() > 0 {
            parts.push(partial);
        }

        let norm_name = parts.iter()
            .map(|word| word.chars().flat_map(|c| c.to_lowercase()).collect::<String>())
            .collect::<Vec<String>>() // have to collect into vec to call join
            .join("_");

        Self {
            header:          header,
            normalized_name: norm_name,
        }
    }
}

#[cfg(test)]
mod field_name_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let nm = FieldName::new("binding");
        assert_eq!(nm.header, None);
        assert_eq!(nm.normalized_name, "binding");
    }

    #[test]
    fn test_camel() {
        let nm = FieldName::new("bindingCount");
        assert_eq!(nm.header, None);
        assert_eq!(nm.normalized_name, "binding_count");
    }

    #[test]
    fn test_stype() {
        let nm = FieldName::new("sType");
        assert_eq!(nm.header, Some(FieldHeader::S));
        assert_eq!(nm.normalized_name, "type");
    }

    #[test]
    fn test_pnext() {
        let nm = FieldName::new("pNext");
        assert_eq!(nm.header, Some(FieldHeader::P));
        assert_eq!(nm.normalized_name, "next");
    }

    #[test]
    fn test_pp() {
        let nm = FieldName::new("ppEnabledLayerNames");
        assert_eq!(nm.header, Some(FieldHeader::PP));
        assert_eq!(nm.normalized_name, "enabled_layer_names");
    }

    #[test]
    fn test_purposes() {
        let nm = FieldName::new("purposes");
        assert_eq!(nm.header, None);
        assert_eq!(nm.normalized_name, "purposes");
    }

    #[test]
    fn test_supported() {
        let nm = FieldName::new("supportedStencilResolveModes");
        assert_eq!(nm.header, None);
        assert_eq!(nm.normalized_name, "supported_stencil_resolve_modes");
    }

    #[test]
    fn test_upcase_word() {
        let nm = FieldName::new("RoundingModeRTEFloat16");
        assert_eq!(nm.header, None);
        assert_eq!(nm.normalized_name, "rounding_mode_rte_float16");
    }
}

pub struct VulkanName {
    pub enum_header:      String,
    pub bit_enum_trailer: String,       // trailer on vk names that are BITS
    pub normalized_name:  String,
    pub ext_name:         Option<String>,
}

impl VulkanName {
    pub fn new(vkname: &str, exts: &HashSet<String>) -> Self {
        // names are of form VkSomethingSomethingOPTEXT
        #[derive(PartialEq)]
        enum Mode {
            V,
            K,
            Ident,
            CapsRun,
        }

        let mut parts = Vec::new();
        let mut partial = String::new();
        let mut last_was_upper = false;

        let mut mode = Mode::V;
        for c in vkname.chars() {
            mode = match mode {
                Mode::V => {
                    if c != 'V' { panic!("Expected 'V' in {}", vkname); }
                    Mode::K
                },
                Mode::K => {
                    if c != 'k' { panic!("Expected 'k' in {}", vkname); }
                    Mode::Ident
                },
                Mode::Ident => {
                    if c.is_uppercase() {
                        if last_was_upper {
                            partial.push(c);
                            Mode::CapsRun
                        }
                        else {
                            if partial.len() > 0 {
                                parts.push(partial);
                                partial = String::new();
                            }
                            partial.push(c);
                            Mode::Ident
                        }
                    }
                    else {
                        partial.push(c);
                        Mode::Ident
                    }
                },
                Mode::CapsRun => {
                    if c.is_uppercase() {
                        partial.push(c);
                        Mode::CapsRun
                    }
                    else {
                        // run is over, switch back.
                        // assume that the previous character is start of *this* word
                        // i.e. VkXYColor => Vk XY Color but partial is XYC right now
                        let pop = partial.pop().unwrap();
                        debug_assert!(pop.is_uppercase());

                        parts.push(partial);
                        partial = String::new();
                        partial.push(pop);
                        partial.push(c);

                        Mode::Ident
                    }
                },
            };

            last_was_upper = c.is_uppercase();
        }

        let mut tail = None;
        if partial.len() != 0 {
            if exts.contains(&partial) {
                tail = Some(partial);
            }
            else {
                parts.push(partial);
            }
        }

        let enum_parts = parts.iter().filter(|&part| {
            part != "Flag" && part != "Bits"
        })
        .map(|s| s.chars().flat_map(|c| c.to_uppercase()).collect::<String>())
        .collect::<Vec<_>>();

        let mut name_parts = parts.iter().filter(|&part| {
            part != "Flag" && part != "Bits"
        })
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        if let Some(ref ext) = &tail {
            name_parts.push(ext.clone());
        }

        let bit_enum_trailer = if let Some(ext) = &tail {
            format!("_BIT_{}", ext)
        }
        else {
            format!("_BIT")
        };

        Self {
            enum_header :     String::from("VK_") + &enum_parts.join("_") + "_",
            bit_enum_trailer: bit_enum_trailer,
            normalized_name:  name_parts.join(""),
            ext_name:         tail,

        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let exts = HashSet::new();
        let name = VulkanName::new("VkQueueFlag", &exts);
        assert_eq!(name.enum_header, "VK_QUEUE_"); // FLAGS is dropped
        assert_eq!(name.bit_enum_trailer, "_BIT");
        assert_eq!(name.normalized_name, "Queue");
    }

    #[test]
    fn test_bits() {
        let exts = HashSet::new();
        let name = VulkanName::new("VkQueueFlagBits", &exts);
        assert_eq!(name.enum_header, "VK_QUEUE_"); // FLAGS is dropped
        assert_eq!(name.bit_enum_trailer, "_BIT");
        assert_eq!(name.normalized_name, "Queue");
    }

    #[test]
    fn test_extension() {
        let mut exts = HashSet::new();
        exts.insert(String::from("KHR"));

        let name = VulkanName::new("VkQueueFlagKHR", &exts);
        assert_eq!(name.enum_header, "VK_QUEUE_"); // FLAGS is dropped
        assert_eq!(name.bit_enum_trailer, "_BIT_KHR");
        assert_eq!(name.normalized_name, "QueueKHR");
        assert_eq!(name.ext_name.unwrap(), "KHR");
    }

    #[test]
    fn test_extension_bits() {
        let mut exts = HashSet::new();
        exts.insert(String::from("KHR"));

        let name = VulkanName::new("VkQueueFlagBitsKHR", &exts);
        assert_eq!(name.enum_header, "VK_QUEUE_"); // FLAGS is dropped
        assert_eq!(name.bit_enum_trailer, "_BIT_KHR");
        assert_eq!(name.normalized_name, "QueueKHR");
        assert_eq!(name.ext_name.unwrap(), "KHR");
    }

    #[test]
    fn test_id() {
        let exts = HashSet::new();
        let name = VulkanName::new("VkSomethingID", &exts);
        assert_eq!(name.enum_header, "VK_SOMETHING_ID_");
        assert_eq!(name.bit_enum_trailer, "_BIT");
        assert_eq!(name.normalized_name, "SomethingID");
        assert_eq!(name.ext_name, None);
    }

    #[test]
    fn test_inner_id() {
        let exts = HashSet::new();
        let name = VulkanName::new("VkSomethingIDElse", &exts);
        assert_eq!(name.enum_header, "VK_SOMETHING_ID_ELSE_");
        assert_eq!(name.normalized_name, "SomethingIDElse");
        assert_eq!(name.ext_name, None);
    }

    #[test]
    fn test_id_ext() {
        let mut exts = HashSet::new();
        exts.insert(String::from("IDK"));

        let name = VulkanName::new("VkSomethingIDK", &exts);
        assert_eq!(name.enum_header, "VK_SOMETHING_");
        assert_eq!(name.bit_enum_trailer, "_BIT_IDK");
        assert_eq!(name.normalized_name, "SomethingIDK");
        assert_eq!(name.ext_name.unwrap(), "IDK");
    }

    #[test]
    fn test_real_thing1() {
        let mut exts = HashSet::new();
        exts.insert(String::from("NV"));

        let name = VulkanName::new("VkGeometryFlagBitsNV", &exts);
        // actual enum VK_GEOMETRY_OPAQUE_BIT_NV
        assert_eq!(name.enum_header, "VK_GEOMETRY_");
        assert_eq!(name.bit_enum_trailer, "_BIT_NV");
        assert_eq!(name.normalized_name, "GeometryNV");
        assert_eq!(name.ext_name.unwrap(), "NV");
    }
}
