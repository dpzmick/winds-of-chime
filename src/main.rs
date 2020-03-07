extern crate winit;
extern crate vk_functions;

mod dynamic_library;
mod macros;
mod sys;

use dynamic_library::DynamicLibrary;
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use std::os::raw::*;

struct Instance<'a> {
    vk:      *mut sys::VkInstance,
    ptrs:    sys::VkInstancePointers,
    phantom: PhantomData<&'a DynamicLibrary>,
}

impl<'a> Instance<'a> {
    fn new(lib: &'a DynamicLibrary) -> Self {
        unsafe {
            // use the minimal loader to boostrap and create an instance
            let ptrs = sys::VkInstanceBootstrap::load(|nm| { lib.sym(nm).unwrap() } );

            let create_info = sys::VkInstanceCreateInfo {
                stype: sys::VkStructureType( sys::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO ),
                next:  null(),
                flags: 0,
                application_info: null(),
                enabled_layer_count: 0,
                enabled_layer_names: null(),
                enabled_extension_count: 0,
                enabled_extension_names: null(),
            };

            let instance = {
                let mut instance: *mut sys::VkInstance = null_mut();

                let result = (ptrs.vkCreateInstance)(&create_info, null(), &mut instance);
                if result.0 != 0 { panic!("failed to create"); }
                instance
            };

            Self {
                vk: instance,
                ptrs: sys::VkInstancePointers::load_with_arg(|nm| lib.sym(nm).unwrap(), instance as *mut c_void),
                phantom: PhantomData
            }
        }

        // then load the full set of function pointers
    }
}

impl<'a> Drop for Instance<'a> {
    fn drop(&mut self) {
        unsafe {
            (self.ptrs.vkDestroyInstance)(self.vk, null());
        }
    }
}

fn main() {
    let lib = DynamicLibrary::open(static_ffi_cstr!("/usr/lib/libvulkan.so")).expect("lib");
    let _instance = Instance::new(&lib);
}
