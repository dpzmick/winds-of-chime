extern crate winit;
extern crate bitflags;

mod test;
mod dynamic_library;
mod macros;
//mod sys;
//mod vk;
mod bf;

use dynamic_library::DynamicLibrary;

fn main() {
    println!("{:?}", test::ShaderStage::ALL_GRAPHICS);
    // let lib = DynamicLibrary::open(static_ffi_cstr!("/usr/lib/libvulkan.so")).expect("lib");

    // let instance = {
    //     let app_info = vk::ApplicationInfoBuilder::new()
    //         .application_name(static_ffi_cstr!("Windows of chime"))
    //         .build();

    //     let layers = &[
    //         static_str_ref!("VK_LAYER_LUNARG_standard_validation"),
    //     ];

    //     let create_info = vk::InstanceCreateInfoBuilder::new()
    //         .application_info(&app_info)
    //         .enabled_layers(layers)
    //         .build();

    //     vk::Instance::new(&lib, &create_info)
    // };

    // // we are just going to use the first physical device
    // let phy = instance.enumerate_physical_devices()[0];

    // // determine what queues we want to connect to
    // // just need a command queue, since we are trying to use cache
    // // coherant memory in round one
    // let queues = instance.get_physical_device_queue_family_properties(phy);
    // for (qidx, q) in queues.iter().enumerate() {
    //     println!("idx {} flags: {}", qidx, q.queueFlags);
    // }

    // for phy in instance.enumerate_physical_devices() {
    //     let props    = instance.get_physical_device_properties(phy);
    //     let features = instance.get_physical_device_features(phy);
    //     let memory   = instance.get_physical_device_memory_properties(phy);

    //     println!("device name: {}", props.device_name());
    //     println!("properties {:?}", props.limits);
    //     println!("features {:?}", instance.get_physical_device_features(phy));
    //     println!("queues: {:?}", instance.get_physical_device_queue_family_properties(phy));
    //     println!("memory types: {:?}", instance.get_physical_device_memory_properties(phy).memory_types());
    //     println!("memory heaps: {:?}", instance.get_physical_device_memory_properties(phy).memory_heaps());
    // }

    // build a shader

    // create a compute pipeline

    // run
}
