#[macro_use]
extern crate ash;
extern crate winit;

#[macro_use]
mod macros;

use std::ffi::CStr;
use std::os::raw::*;

// traits
use ash::version::{EntryV1_0, InstanceV1_0, DeviceV1_0};

// FIXME figure out how to "store" this with the right type, what is the layout anyway?
const MEMCPY_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/shaders/memcpy.spirv"));

fn get_memcpy_code() -> &'static [u32] // hopefully a glorified cast
{
    let ptr = MEMCPY_CODE.as_ptr() as *const u32;
    let len = MEMCPY_CODE.len()/4;
    return unsafe { std::slice::from_raw_parts(ptr, len) };
}

unsafe extern "system" fn vulkan_debug_callback(
    _: ash::vk::DebugReportFlagsEXT,
    _: ash::vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    ash::vk::FALSE
}

fn main() {
    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_title("Triangle")
        .with_dimensions(winit::dpi::LogicalSize::new(600., 800.))
        .build(&events_loop)
        .expect("Failed to create window");

    let lib = ash::Entry::new().unwrap(); // lib loader
    let instance = {
        let app_info = ash::vk::ApplicationInfo::builder()
            .application_name(ffi_cstr!("Hello Triangle"))
            .application_version(vk_make_version!(1, 0, 0))
            .engine_name(ffi_cstr!("dpzmick"))
            .engine_version(vk_make_version!(1, 0, 0))
            .api_version(vk_make_version!(1, 0, 0));

        let layer_names = &[
            ffi_cstr!("VK_LAYER_LUNARG_standard_validation").as_ptr(),
        ];

        let ext_names = &[
            //Surface::name().as_ptr(),
            //XlibSurface::name().as_ptr(),
            ash::extensions::ext::DebugReport::name().as_ptr(),
        ];

        let create_info = ash::vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(layer_names)
            .enabled_extension_names(ext_names);

        unsafe { lib.create_instance(&create_info, None) }.expect("Failed to create instance")
    };

}

// fn old_main() {
//     let debug_info = ash::vk::DebugReportCallbackCreateInfoEXT::builder()
//         .flags(
//             ash::vk::DebugReportFlagsEXT::ERROR
//             | ash::vk::DebugReportFlagsEXT::WARNING
//             | ash::vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
//         )
//         .pfn_callback(Some(vulkan_debug_callback));

//     let debug_report_loader = ash::extensions::ext::DebugReport::new(&lib, &instance);
//     let _debug_call_back = unsafe { debug_report_loader
//         .create_debug_report_callback(&debug_info, None)
//         .unwrap() };

//     let mut device           = None; // copy of handle
//     let mut device_name      = None;
//     let mut device_features  = None;
//     let mut queue_family_idx = None;
//     let mut mem_type_idx     = None;

//     let physical_devices = unsafe { instance.enumerate_physical_devices().expect("No devices found") };
//     println!("{} Devices:", physical_devices.len());
//     for dev in &physical_devices { // FIXME dev is a ref, but doesn't need to be
//         let props = unsafe { instance.get_physical_device_properties(*dev) };
//         let name  = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
//         let feats = unsafe { instance.get_physical_device_features(*dev) };
//         println!("  Name: {:?} Type: {:?}", name, props.device_type);

//         let mut qidx   = None;
//         let mut memidx = None;

//         let queue_props = unsafe { instance.get_physical_device_queue_family_properties(*dev) };
//         println!("    {} queue families:", queue_props.len());
//         for (idx, q) in queue_props.iter().enumerate() {
//             println!("    flags: {:?}",   q.queue_flags);
//             println!("      n_queue: {:?}", q.queue_count);

//             if q.queue_flags.intersects(ash::vk::QueueFlags::COMPUTE) {
//                 qidx = Some(idx as u32)
//             }
//         }

//         let mem_props = unsafe { instance.get_physical_device_memory_properties(*dev) };
//         println!("    {} memory types", mem_props.memory_type_count);
//         for idx in 0..mem_props.memory_type_count {
//             let mem = mem_props.memory_types[idx as usize];
//             println!("      {:?}", mem);

//             use ash::vk::MemoryPropertyFlags;
//             let want = MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT;
//             if mem.property_flags.intersects(want) {
//                 memidx = Some(idx);
//             }

//             // FIXME check size
//         }

//         if device.is_none() && !qidx.is_none() && !memidx.is_none() {
//             device           = Some(*dev);
//             device_name      = Some(name.to_str().expect("Bad device name"));
//             device_features  = Some(feats); // copy
//             queue_family_idx = qidx;
//             mem_type_idx     = memidx;
//         }
//     }

//     if device.is_none() { panic!("No suitable device found") }

//     println!("\nPicked Device: '{}'. Queue family idx: {}. Mem type idx: {}",
//              device_name.unwrap(),
//              queue_family_idx.unwrap(),
//              mem_type_idx.unwrap());

//     // only planning on creating a single queue
//     let queue_prio: [f32; 1] = [0.0];

//     let dev = {
//         let queue_create_infos = [
//             ash::vk::DeviceQueueCreateInfo::builder()
//                 .flags(ash::vk::DeviceQueueCreateFlags::empty())
//                 .queue_family_index(queue_family_idx.unwrap())
//                 .queue_priorities(&queue_prio)    // number of queues determined from size of prio
//                 .build(),                         // calling build not ideal, careful
//         ];

//         let create_info = ash::vk::DeviceCreateInfo::builder()
//             .flags(ash::vk::DeviceCreateFlags::empty())
//             .queue_create_infos(&queue_create_infos)
//             .enabled_layer_names(&[])
//             .enabled_extension_names(&[])
//             .enabled_features(device_features.as_ref().unwrap());

//         unsafe { instance.create_device(device.unwrap(), &create_info, None) }.expect("Failed to create device")
//     };

//     let queue = unsafe { dev.get_device_queue(queue_family_idx.unwrap(), 0) };

//     let shader_module = {
//         let create_info = ash::vk::ShaderModuleCreateInfo::builder()
//             .flags(ash::vk::ShaderModuleCreateFlags::empty())
//             .code(get_memcpy_code());

//         unsafe { dev.create_shader_module(&create_info, None) }.expect("Failed to create shader module")
//     };

//     // Allocate memory
//     let shm_handle = {
//         let create_info = ash::vk::MemoryAllocateInfo::builder()
//             .allocation_size(1024)
//             .memory_type_index(mem_type_idx.unwrap());

//         unsafe { dev.allocate_memory(&create_info, None) }.expect("Failed to allocate memory")
//     };

//     let mut tmp = Vec::new();
//     unsafe {
//         let ptr = dev.map_memory(shm_handle, 0, 1024, ash::vk::MemoryMapFlags::empty()).expect("Failed to map memory");
//         let slice = std::slice::from_raw_parts_mut(ptr as *mut u32, 1024/std::mem::size_of::<u32>());
//         for (i, x) in slice.iter_mut().enumerate() {
//             *x = i as u32;
//             tmp.push(*x);
//         }

//         dev.unmap_memory(shm_handle);
//     };

//     let in_buffer = {
//         let tmp = [queue_family_idx.unwrap()];
//         let create_info = ash::vk::BufferCreateInfo::builder()
//             // flags default
//             .size(512)
//             .usage(ash::vk::BufferUsageFlags::STORAGE_BUFFER)
//             .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
//             .queue_family_indices(&tmp);

//         unsafe { dev.create_buffer(&create_info, None) }.expect("Failed to create buffer")
//     };

//     let out_buffer = {
//         let tmp = [queue_family_idx.unwrap()];
//         let create_info = ash::vk::BufferCreateInfo::builder()
//             // flags default
//             .size(512)
//             .usage(ash::vk::BufferUsageFlags::STORAGE_BUFFER)
//             .sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
//             .queue_family_indices(&tmp);

//         unsafe { dev.create_buffer(&create_info, None) }.expect("Failed to create buffer")
//     };

//     unsafe { dev.bind_buffer_memory(in_buffer,  shm_handle, 0)   }.expect("Failed to bind");
//     unsafe { dev.bind_buffer_memory(out_buffer, shm_handle, 512) }.expect("Failed to bind");

//     let dsetlayout = {
//         let bindings = &[
//             ash::vk::DescriptorSetLayoutBinding::builder()
//                 .binding(0)
//                 .descriptor_type(ash::vk::DescriptorType::STORAGE_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(ash::vk::ShaderStageFlags::COMPUTE)
//                 .build(),
//             ash::vk::DescriptorSetLayoutBinding::builder()
//                 .binding(1)
//                 .descriptor_type(ash::vk::DescriptorType::STORAGE_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(ash::vk::ShaderStageFlags::COMPUTE)
//                 .build(),
//         ];
//         let create_info = ash::vk::DescriptorSetLayoutCreateInfo::builder()
//             .flags(ash::vk::DescriptorSetLayoutCreateFlags::empty())
//             .bindings(bindings);

//         unsafe { dev.create_descriptor_set_layout(&create_info, None) }.expect("Failed to create dset layout")
//     };

//     let dpool = {
//         let sizes = &[
//             ash::vk::DescriptorPoolSize::builder()
//                 .ty(ash::vk::DescriptorType::STORAGE_BUFFER)
//                 .descriptor_count(2)
//                 .build(),
//         ];
//         let create_info = ash::vk::DescriptorPoolCreateInfo::builder()
//             .max_sets(1)
//             .pool_sizes(sizes);

//         unsafe { dev.create_descriptor_pool(&create_info, None) }.expect("Failed to create descriptor pool")
//     };

//     let dset = {
//         let layouts = &[dsetlayout];
//         let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
//             .descriptor_pool(dpool)
//             .set_layouts(layouts);

//         unsafe { dev.allocate_descriptor_sets(&alloc_info) }.expect("Failed to allocate descriptor sets")[0]
//     };

//     // unrelated to the other dset layouts and stuff?
//     // where does it make the most sense to do this?
//     {
//         let in_info = &[
//             ash::vk::DescriptorBufferInfo::builder()
//                 .buffer(in_buffer)
//                 .offset(0)
//                 .range(ash::vk::WHOLE_SIZE)
//                 .build()
//         ];

//         let out_info = &[
//             ash::vk::DescriptorBufferInfo::builder()
//                 .buffer(out_buffer)
//                 .offset(0)
//                 .range(ash::vk::WHOLE_SIZE)
//                 .build()
//         ];

//         let write_descs = &[
//             ash::vk::WriteDescriptorSet::builder()
//                 .dst_set(dset)
//                 .dst_binding(0)
//                 .descriptor_type(ash::vk::DescriptorType::STORAGE_BUFFER)
//                 .buffer_info(in_info)
//                 .build(),

//             ash::vk::WriteDescriptorSet::builder()
//                 .dst_set(dset)
//                 .dst_binding(1)
//                 .descriptor_type(ash::vk::DescriptorType::STORAGE_BUFFER)
//                 .buffer_info(out_info)
//                 .build(),
//         ];

//         unsafe { dev.update_descriptor_sets(write_descs, &[]) };
//     }

//     let pipeline_layout = {
//         let layouts = [dsetlayout]; // lifetime okay?
//         let create_info = ash::vk::PipelineLayoutCreateInfo::builder()
//             .flags(ash::vk::PipelineLayoutCreateFlags::empty())
//             .set_layouts(&layouts);

//         unsafe { dev.create_pipeline_layout(&create_info, None) }.expect("Failed to create pipleline layout")
//     };

//     let pipeline = {
//         let shader_stage_create_info = ash::vk::PipelineShaderStageCreateInfo::builder()
//             .stage(ash::vk::ShaderStageFlags::COMPUTE)
//             .module(shader_module)
//             .name(ffi_cstr!("main"));

//         let create_info = ash::vk::ComputePipelineCreateInfo::builder()
//             .stage(*shader_stage_create_info)
//             .layout(pipeline_layout);

//         unsafe { dev.create_compute_pipelines(ash::vk::PipelineCache::null(), &[*create_info], None) }.expect("Failed to create pipeline")[0]
//     };

//     let cpool = {
//         let create_info = ash::vk::CommandPoolCreateInfo::builder()
//             .flags(ash::vk::CommandPoolCreateFlags::empty())
//             .queue_family_index(queue_family_idx.unwrap());

//         unsafe { dev.create_command_pool(&create_info, None) }.expect("Failed to create command pool")
//     };

//     let cbuffer = {
//         let create_info = ash::vk::CommandBufferAllocateInfo::builder()
//             .command_pool(cpool)
//             .level(ash::vk::CommandBufferLevel::PRIMARY)
//             .command_buffer_count(1);

//         unsafe { dev.allocate_command_buffers(&create_info) }.expect("Failed to create command buffers")[0]
//     };

//     // build cbuffer
//     {
//         let create_info = ash::vk::CommandBufferBeginInfo::builder()
//             .flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

//         unsafe { dev.begin_command_buffer(cbuffer, &create_info) }.expect("Failed to begin command buffer");
//         unsafe { dev.cmd_bind_pipeline(cbuffer, ash::vk::PipelineBindPoint::COMPUTE, pipeline) };
//         unsafe { dev.cmd_bind_descriptor_sets(cbuffer, ash::vk::PipelineBindPoint::COMPUTE, pipeline_layout, 0, &[dset], &[]) };
//         unsafe { dev.cmd_dispatch(cbuffer, 512, 512, 1) };
//         unsafe { dev.end_command_buffer(cbuffer) }.expect("Failed to end command buffer");
//     }

//     let cbuffers = &[cbuffer];
//     let submit_info = ash::vk::SubmitInfo::builder()
//         .command_buffers(cbuffers);

//     unsafe { dev.queue_submit(queue, &[submit_info.build()], ash::vk::Fence::null()) }.expect("Failed to submit");
//     unsafe { dev.queue_wait_idle(queue) }.expect("Failed to wait until idle");

//     unsafe {
//         let mut tmp2 = Vec::new();
//         let ptr = dev.map_memory(shm_handle, 0, 1024, ash::vk::MemoryMapFlags::empty()).expect("Failed to map memory");
//         let slice = std::slice::from_raw_parts_mut(ptr as *mut u32, 1024/std::mem::size_of::<u32>());
//         for x in slice {
//             tmp2.push(*x);
//         }

//         println!("before:");
//         println!("{:?}", tmp);
//         println!("\nafter:");
//         println!("{:?}", tmp2);

//         if tmp != tmp2 {
//             panic!("something went wrong");
//         }
//     }

//     // have to cleanup manually...
//     unsafe { dev.free_command_buffers(cpool, &[cbuffer]) };
//     unsafe { dev.destroy_command_pool(cpool, None) };
//     unsafe { dev.destroy_pipeline_layout(pipeline_layout, None) };
//     unsafe { dev.destroy_descriptor_set_layout(dsetlayout, None) };
//     unsafe { dev.destroy_buffer(out_buffer, None) };
//     unsafe { dev.destroy_buffer(in_buffer, None) };
//     unsafe { dev.free_memory(shm_handle, None) };
//     unsafe { dev.destroy_shader_module(shader_module, None) };
//     unsafe { dev.destroy_device(None) };
//     unsafe { instance.destroy_instance(None) };

// }
