extern crate vk_functions;
use vk_functions::vk_functions;

// vk_handles!{
//     VkInstance,
// }

struct VkInstance {
    _private: [u8; 0]
}

vk_functions!(
    struct VkInstancePointers
    loader VkGetInstanceProcAddress

    // return types are required
    fn VkCreateInstance(instance: *mut VkInstance, asd: u32) -> VkResult;
    fn VkDestroyInstance(instance: *mut VkInstance, dsa: u32) -> ();
);

#[test]
fn main() {
    // let pointers = VkInstancePointers::load(|name| {
    //     lib.sym(name)
    // });
}
