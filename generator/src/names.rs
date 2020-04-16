use std::collections::HashSet;

#[derive(Debug)]
pub struct VulkanName {
    pub enum_header:      String,
    pub bit_enum_trailer: String,       // trailer on vk names that are BITS
    pub normalized_name:  String,
    pub ext_name:         Option<String>,
}

impl VulkanName {
    // FIXME hashset should be of &str?
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
