#[macro_use]
extern crate ash;
extern crate winit;

#[macro_use]
mod macros;

use std::ffi::CStr;

// traits
use ash::version::{EntryV1_0, InstanceV1_0, DeviceV1_0};

// FIXME store shaders SPIR-V in static part of executable, or store in special output dir?

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

    let mut device           = None; // copy of handle
    let mut device_name      = None;
    let mut device_features  = None;
    let mut queue_family_idx = None;
    let mut mem_type_idx     = None;

    let physical_devices = unsafe { instance.enumerate_physical_devices().expect("No devices found") };
    println!("{} Devices:", physical_devices.len());
    for dev in &physical_devices { // FIXME dev is a ref, but doesn't need to be
        let props = unsafe { instance.get_physical_device_properties(*dev) };
        let name  = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
        let feats = unsafe { instance.get_physical_device_features(*dev) };
        println!("  Name: {:?} Type: {:?}", name, props.device_type);

        let mut qidx   = None;
        let mut memidx = None;

        let queue_props = unsafe { instance.get_physical_device_queue_family_properties(*dev) };
        println!("    {} queue families:", queue_props.len());
        for (idx, q) in queue_props.iter().enumerate() {
            println!("    flags: {:?}",   q.queue_flags);
            println!("      n_queue: {:?}", q.queue_count);

            if q.queue_flags.intersects(ash::vk::QueueFlags::COMPUTE) {
                qidx = Some(idx as u32)
            }
        }

        let mem_props = unsafe { instance.get_physical_device_memory_properties(*dev) };
        println!("    {} memory types", mem_props.memory_type_count);
        for idx in 0..mem_props.memory_type_count {
            let mem = mem_props.memory_types[idx as usize];
            println!("      {:?}", mem);

            use ash::vk::MemoryPropertyFlags;
            let want = MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT;
            if mem.property_flags.intersects(want) {
                memidx = Some(idx);
            }

            // FIXME check size
        }

        if device.is_none() && !qidx.is_none() && !memidx.is_none() {
            device           = Some(*dev);
            device_name      = Some(name.to_str().expect("Bad device name"));
            device_features  = Some(feats); // copy
            queue_family_idx = qidx;
            mem_type_idx     = memidx;
        }
    }

    if device.is_none() { panic!("No suitable device found") }

    println!("\nPicked Device: '{}'. Queue family idx: {}. Mem type idx: {}",
             device_name.unwrap(),
             queue_family_idx.unwrap(),
             mem_type_idx.unwrap());

    // only planning on creating a single queue
    let queue_prio: [f32; 1] = [0.0];

    let dev = {
        let queue_create_infos = [
            ash::vk::DeviceQueueCreateInfo::builder()
                .flags(ash::vk::DeviceQueueCreateFlags::empty())
                .queue_family_index(queue_family_idx.unwrap())
                .queue_priorities(&queue_prio)    // number of queues determined from size of prio
                .build(),                         // calling build not ideal, careful
        ];

        let create_info = ash::vk::DeviceCreateInfo::builder()
            .flags(ash::vk::DeviceCreateFlags::empty())
            .queue_create_infos(&queue_create_infos)
            .enabled_layer_names(&[])
            .enabled_extension_names(&[])
            .enabled_features(device_features.as_ref().unwrap());

        unsafe { instance.create_device(device.unwrap(), &create_info, None) }.expect("Failed to create device")
    };

    let queue = unsafe { dev.get_device_queue(queue_family_idx.unwrap(), 0) };

    // Allocate memory
    let shm_handle = {
        let create_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(1024)
            .memory_type_index(mem_type_idx.unwrap());

        unsafe { dev.allocate_memory(&create_info, None) }.expect("Failed to allocate memory")
    };

    let in_buffer = {
        let tmp = [queue_family_idx.unwrap()];
        let create_info = ash::vk::BufferCreateInfo::builder()
            // flags default
            .size(512)
            .usage(ash::vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&tmp);

        unsafe { dev.create_buffer(&create_info, None) }.expect("Failed to create buffer")
    };

    let out_buffer = {
        let tmp = [queue_family_idx.unwrap()];
        let create_info = ash::vk::BufferCreateInfo::builder()
            // flags default
            .size(512)
            .usage(ash::vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&tmp);

        unsafe { dev.create_buffer(&create_info, None) }.expect("Failed to create buffer")
    };

    unsafe { dev.bind_buffer_memory(in_buffer,  shm_handle, 0)   }.expect("Failed to bind");
    unsafe { dev.bind_buffer_memory(out_buffer, shm_handle, 512) }.expect("Failed to bind");

    // can now map/unmap the regions
    // create shader
    // create compute pipeline
    // create command pool
    // allocate command buffer
    // save a shader dispatch on command buffer
    // submit to queue
    // ???? wait
    // read results out of mapped memory

    // have to cleanup manually...
    unsafe { dev.destroy_buffer(out_buffer, None) };
    unsafe { dev.destroy_buffer(in_buffer, None) };
    unsafe { dev.free_memory(shm_handle, None) };
    unsafe { dev.destroy_device(None) };
    unsafe { instance.destroy_instance(None) };
}
