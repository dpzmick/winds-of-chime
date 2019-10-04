#[macro_use]
extern crate ash;
extern crate winit;

#[macro_use]
mod macros;

use std::ffi::CStr;

// Some Traits
use ash::version::{EntryV1_0, InstanceV1_0};

fn main() {
    let lib = ash::Entry::new().unwrap(); // lib loader
    let instance = {
        let app_info = ash::vk::ApplicationInfo::builder()
            .application_name(ffi_cstr!("Hello Triangle"))
            .application_version(vk_make_version!(1, 0, 0))
            .engine_name(ffi_cstr!("dpzmick"))
            .engine_version(vk_make_version!(1, 0, 0))
            .api_version(vk_make_version!(1, 0, 0));

        // FIXME validation layers
        let create_info = ash::vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&[])
            .enabled_extension_names(&[]);

        unsafe { lib.create_instance(&create_info, None) }.expect("Failed to create instance")
    };

    let physical_devices = unsafe { instance.enumerate_physical_devices().expect("No devices found") };
    println!("Devices:");
    for dev in physical_devices.iter() {
        let props = unsafe { instance.get_physical_device_properties(*dev) };
        println!("Name: {:?} Type: {:?}",
                 unsafe { CStr::from_ptr(props.device_name.as_ptr()) },
                 props.device_type)
    }
}
