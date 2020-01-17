#[macro_use]
extern crate ash;
extern crate winit;
extern crate glm;

#[macro_use]
mod macros;

use std::ffi::CStr;
use std::os::raw::*;

// traits
use ash::version::{EntryV1_0, InstanceV1_0, DeviceV1_0};

// I don't think layout matters, since we bnd explicit regions to the card
#[derive(Clone)]
struct Vertex {
    pos:   glm::Vector2<f32>,
    color: glm::Vector3<f32>,
}

impl Vertex {
    fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Vertex {
            pos:   glm::Vector2::new(pos[0], pos[1]),
            color: glm::Vector3::new(color[0], color[1], color[2]),
        }
    }

    fn make_bindings_desc() -> ash::vk::VertexInputBindingDescription {
        ash::vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(ash::vk::VertexInputRate::VERTEX)
            .build()
    }

    fn make_attr_desc() -> [ash::vk::VertexInputAttributeDescription; 2] {
        let inst = Vertex { // FIXME hacking around lack of offsetof
            pos: glm::Vector2::new(1., 1.),
            color: glm::Vector3::new(1., 1., 1.),
        };

        [ash::vk::VertexInputAttributeDescription::builder()
         .binding(0)
         .location(0)
         .format(ash::vk::Format::R32G32_SFLOAT)
         .offset(((&inst.pos as *const _ as usize) - (&inst as *const _ as usize)) as u32)
         .build(),
         ash::vk::VertexInputAttributeDescription::builder()
         .binding(0)
         .location(1)
         .format(ash::vk::Format::R32G32B32_SFLOAT)
         .offset(((&inst.color as *const _ as usize) - (&inst as *const _ as usize)) as u32)
         .build()]
    }
}

fn get_vert_code() -> &'static [u32]
{
    const CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/shaders/vert.spirv"));
    let ptr = CODE.as_ptr() as *const u32;
    let len = CODE.len()/4;
    return unsafe { std::slice::from_raw_parts(ptr, len) };
}

fn get_frag_code() -> &'static [u32]
{
    const CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/shaders/frag.spirv"));
    let ptr = CODE.as_ptr() as *const u32;
    let len = CODE.len()/4;
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
    let surface_resolution = ash::vk::Extent2D {
        // width: window.get_inner_size().unwrap().width as u32,
        // height: window.get_inner_size().unwrap().height as u32,
        // width: 2304,
        // height: 1728,
        width: 800,
        height: 800,
    };

    // square
    let vertex_data = [
        Vertex::new([-0.5, -0.5], [1.0, 0.0, 0.0]),
        Vertex::new([ 0.5, -0.5], [0.0, 1.0, 0.0]),
        Vertex::new([ 0.5,  0.5], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5,  0.5], [1.0, 1.0, 1.0]),
    ];

    let indicies: &[u16] = &[0, 1, 2, 2, 3, 0];

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Triangle")
        .with_inner_size(winit::dpi::LogicalSize::new(
            surface_resolution.width as f64,
            surface_resolution.height as f64))
        .with_min_inner_size(winit::dpi::LogicalSize::new(
            surface_resolution.width as f64,
            surface_resolution.height as f64))
        .with_max_inner_size(winit::dpi::LogicalSize::new(
            surface_resolution.width as f64,
            surface_resolution.height as f64))
        .with_resizable(false)
        .build(&event_loop)
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
            ash::extensions::ext::DebugReport::name().as_ptr(),
            ash::extensions::khr::Surface::name().as_ptr(),
            // ash::extensions::khr::WaylandSurface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
        ];

        let create_info = ash::vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(layer_names)
            .enabled_extension_names(ext_names);

        unsafe { lib.create_instance(&create_info, None) }.expect("Failed to create instance")
    };

    let debug_info = ash::vk::DebugReportCallbackCreateInfoEXT::builder()
        .flags(
            ash::vk::DebugReportFlagsEXT::ERROR
            | ash::vk::DebugReportFlagsEXT::WARNING
            | ash::vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
        )
        .pfn_callback(Some(vulkan_debug_callback));

    let debug_report_loader = ash::extensions::ext::DebugReport::new(&lib, &instance);
    let _debug_call_back = unsafe {
        debug_report_loader
            .create_debug_report_callback(&debug_info, None)
            .unwrap()
    };

    let surface_loader = ash::extensions::khr::Surface::new(&lib, &instance);
    // let surface = {
    //     use winit::platform::unix::WindowExtUnix;
    //     let disp = window.wayland_display().expect("Couldn't get wayland display");
    //     let surf = window.wayland_surface().expect("Couldn't get wayland surface");
    //     let create_info = ash::vk::WaylandSurfaceCreateInfoKHR::builder()
    //         .surface(surf)
    //         .display(disp);

    //     let ext = ash::extensions::khr::WaylandSurface::new(&lib, &instance);
    //     unsafe { ext.create_wayland_surface(&create_info, None) }.expect("Failed to create surface")
    // };
    let surface = {
        use winit::platform::unix::WindowExtUnix;
        let disp = window.xlib_display().expect("Couldn't get xlib display");
        let win  = window.xlib_window().expect("Couldn't get xlib window");
        let create_info = ash::vk::XlibSurfaceCreateInfoKHR::builder()
            .window(win)
            .dpy(disp as *mut ash::vk::Display);

        let ext = ash::extensions::khr::XlibSurface::new(&lib, &instance);
        unsafe { ext.create_xlib_surface(&create_info, None) }.expect("Failed to create surface")
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
            let present = unsafe {
                surface_loader.get_physical_device_surface_support(*dev, idx as u32, surface)
            };

            println!("    flags: {:?}",   q.queue_flags);
            println!("      n_queue: {:?}", q.queue_count);
            println!("      present: {:?}", present);

            // can't quite do exactly what the tutorial specifies on
            // my intel graphics. I'm getting:
            // 1 Devices:
            // Name: "Intel(R) UHD Graphics 620 (Kabylake GT2)" Type: INTEGRATED_GPU
            //   1 queue families:
            //   flags: GRAPHICS | COMPUTE | TRANSFER
            //     n_queue: 1
            //     present: true
            //   2 memory families:
            //       MemoryType { property_flags: DEVICE_LOCAL | HOST_VISIBLE | HOST_COHERENT | HOST_CACHED, heap_index: 0 }
            //       MemoryType { property_flags: DEVICE_LOCAL | HOST_VISIBLE | HOST_COHERENT | HOST_CACHED, heap_index: 1 }
            // so instead, going to try and use mutex and signaling to do the transfer
            // I'm going to skip this part of the tutorial for now..

            let want = ash::vk::QueueFlags::GRAPHICS | ash::vk::QueueFlags::TRANSFER;
            if q.queue_flags.intersects(want) && present {
                qidx = Some(idx as u32);
            }
        }

        let mem_props = unsafe { instance.get_physical_device_memory_properties(*dev) };
        println!("    {} memory families:", mem_props.memory_type_count);
        for idx in 0..mem_props.memory_type_count {
            let mem = mem_props.memory_types[idx as usize];
            println!("        {:?}", mem);

            use ash::vk::MemoryPropertyFlags;
            let want = MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT;
            if mem.property_flags.intersects(want) {
                memidx = Some(idx);
            }
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

    let surface_format = {
        let fmt = unsafe { surface_loader.get_physical_device_surface_formats(device.unwrap(), surface) }
            .expect("Failed to get surface formats");

        fmt
            .iter()
            .nth(0)
            .expect("No color formats found")
            .clone()
    };

    let surface_caps = unsafe {
        surface_loader.get_physical_device_surface_capabilities(device.unwrap(), surface)
    }.expect("Failed to get surface capabilites");

    let image_cnt = if surface_caps.max_image_count > 0 {
        std::cmp::max(surface_caps.min_image_count+1, surface_caps.max_image_count)
    }
    else {
        surface_caps.min_image_count+1
    };

    // just going for defaults here
    let pre_transform = ash::vk::SurfaceTransformFlagsKHR::IDENTITY;
    let present_mode = ash::vk::PresentModeKHR::FIFO; // always must exist

    println!("\nPicked Device: '{}'.", device_name.unwrap());
    println!("    Queue family idx: {}.", queue_family_idx.unwrap());
    println!("    Surface format: {:?}.", surface_format);
    println!("    Surface resolution: {:?}", surface_resolution);
    println!("    Image count {}", image_cnt);
    println!("    Present mode {:?}", present_mode);

    // only planning on creating a single queue
    let queue_prio: [f32; 1] = [0.0]; // must live for entire lifetime of application
    let dev = {
        let queue_create_infos = [
            ash::vk::DeviceQueueCreateInfo::builder()
                .flags(ash::vk::DeviceQueueCreateFlags::empty())
                .queue_family_index(queue_family_idx.unwrap())
                .queue_priorities(&queue_prio)    // number of queues determined from size of prio
                .build(),                         // calling build not ideal, careful
        ];

        let extension_names = &[
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ];

        let create_info = ash::vk::DeviceCreateInfo::builder()
            .flags(ash::vk::DeviceCreateFlags::empty())
            .queue_create_infos(&queue_create_infos)
            .enabled_layer_names(&[])
            .enabled_extension_names(extension_names)
            .enabled_features(device_features.as_ref().unwrap());

        unsafe { instance.create_device(device.unwrap(), &create_info, None) }
        .expect("Failed to create device")
    };

    let queue = unsafe { dev.get_device_queue(queue_family_idx.unwrap(), 0) };

    let vertex_buffer = {
        let create_info = ash::vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<Vertex>() as u64) * (vertex_data.len() as u64))
            .usage(ash::vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        unsafe { dev.create_buffer(&create_info, None) }
    }.expect("Failed to create buffer");

    let mem_reqs = unsafe { dev.get_buffer_memory_requirements(vertex_buffer) };

    let vertex_memory = {
        let alloc_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_reqs.size)
            .memory_type_index(mem_type_idx.unwrap());

        unsafe { dev.allocate_memory(&alloc_info, None) }
    }.expect("Failed to allocate vertex memory");

    unsafe { dev.bind_buffer_memory(vertex_buffer, vertex_memory, 0) }.expect("Failed to bind");

    unsafe {
        let ptr = dev.map_memory(vertex_memory, 0, mem_reqs.size, ash::vk::MemoryMapFlags::empty()).expect("Failed to map memory");
        let slice = std::slice::from_raw_parts_mut(ptr as *mut Vertex,
                                                   (mem_reqs.size as usize) / std::mem::size_of::<Vertex>());
        for (i, x) in slice.iter_mut().enumerate() {
            *x = vertex_data[i].clone();
        }
    };

    let index_buffer = {
        let create_info = ash::vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<u16>() as u64) * (indicies.len() as u64))
            .usage(ash::vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        unsafe { dev.create_buffer(&create_info, None) }
    }.expect("Failed to create index buffer");

    let index_memory = {
        let alloc_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_reqs.size)
            .memory_type_index(mem_type_idx.unwrap());

        unsafe { dev.allocate_memory(&alloc_info, None) }
    }.expect("Failed to allocate index memory");

    unsafe { dev.bind_buffer_memory(index_buffer, index_memory, 0) }.expect("Failed to bind");

    unsafe {
        let ptr = dev.map_memory(index_memory, 0, mem_reqs.size, ash::vk::MemoryMapFlags::empty()).expect("Failed to map memory");
        let slice = std::slice::from_raw_parts_mut(ptr as *mut u16,
                                                   (mem_reqs.size as usize) / std::mem::size_of::<u16>());

        for (i, x) in indicies.iter().enumerate() {
            slice[i] = x.clone();
        }
    };

    let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &dev);
    let swapchain = {
        let actual_extent = ash::vk::Extent2D {
            width: std::cmp::max(
                surface_caps.min_image_extent.width,
                std::cmp::min(
                    surface_caps.max_image_extent.width,
                    surface_resolution.width
                )
            ),
            height: std::cmp::max(
                surface_caps.min_image_extent.height,
                std::cmp::min(
                    surface_caps.max_image_extent.height,
                    surface_resolution.height
                )
            ),
        };
        let create_info = ash::vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_cnt)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(actual_extent)
            .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .image_array_layers(1);

        unsafe { swapchain_loader.create_swapchain(&create_info, None) }
    }.expect("Failed to create swapchain");

    let present_views: Vec<ash::vk::ImageView> =
        unsafe { swapchain_loader.get_swapchain_images(swapchain) }
        .expect("Failed to get swapchain images")
            .iter()
            .map(|&image| {
                let create_info = ash::vk::ImageViewCreateInfo::builder()
                    .view_type(ash::vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(ash::vk::ComponentMapping {
                        r: ash::vk::ComponentSwizzle::IDENTITY,
                        g: ash::vk::ComponentSwizzle::IDENTITY,
                        b: ash::vk::ComponentSwizzle::IDENTITY,
                        a: ash::vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(ash::vk::ImageSubresourceRange {
                        aspect_mask:      ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level:   0,
                        level_count:      1,
                        base_array_layer: 0,
                        layer_count:      1,
                    })
                    .image(image);

                unsafe { dev.create_image_view(&create_info, None) }
                    .expect("Failed to create image view")
            })
            .collect();

    let pipeline_layout = {
        let create_info = ash::vk::PipelineLayoutCreateInfo::builder();
        // leave empty
        unsafe { dev.create_pipeline_layout(&create_info, None) }
    }.expect("Failed to create pipeline layout");

    let render_pass = {
        let attachments = &[
            ash::vk::AttachmentDescription::builder()
                .format(surface_format.format)
                .samples(ash::vk::SampleCountFlags::TYPE_1)
                .load_op(ash::vk::AttachmentLoadOp::CLEAR)
                .store_op(ash::vk::AttachmentStoreOp::STORE)
                .stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(ash::vk::ImageLayout::UNDEFINED)
                .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
        ];

        let refs = &[
            ash::vk::AttachmentReference::builder()
                .attachment(0)
                .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build(),
        ];

        let subpasses = &[
            ash::vk::SubpassDescription::builder()
                .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(refs)
                .build()
        ];

        let create_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses);

        unsafe { dev.create_render_pass(&create_info, None) }
    }.expect("Failed to create render pass");

    let graphics_pipeline = {
        let vert = {
            let create_info = ash::vk::ShaderModuleCreateInfo::builder()
                .code(get_vert_code());
            unsafe { dev.create_shader_module(&create_info, None) }
        }.expect("Failed to create vertex shader module");

        let frag = {
            let create_info = ash::vk::ShaderModuleCreateInfo::builder()
                .code(get_frag_code());
            unsafe { dev.create_shader_module(&create_info, None) }
        }.expect("Failed to create fragment shader module");

        let stages = &[
            ash::vk::PipelineShaderStageCreateInfo::builder()
                .stage(ash::vk::ShaderStageFlags::VERTEX)
                .module(vert)
                .name(ffi_cstr!("main"))
                .build(),
            ash::vk::PipelineShaderStageCreateInfo::builder()
                .stage(ash::vk::ShaderStageFlags::FRAGMENT)
                .module(frag)
                .name(ffi_cstr!("main"))
                .build(),
        ];

        let bindings = [Vertex::make_bindings_desc()];
        let attrs    = Vertex::make_attr_desc();

        // describes where to get vertexices from (and what the
        // bindings should be)
        let input_state = ash::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attrs);

        // what kind of geometry to graw from the verticies
        // if primitive restart should be enabled (whatever that means)
        let input_assembly = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewports = &[
            ash::vk::Viewport {
                x:        0.0,
                y:        0.0,
                width:    surface_resolution.width  as f32,
                height:   surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }
        ];

        let scissors = &[
            ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: surface_resolution.clone()
            }
        ];

        // don't actually understand this yet.. ???
        let viewport = ash::vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);

        let raster = ash::vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample = ash::vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

        // explain how to combine colors produced by fragment shader
        // with colors already in framebuffer
        let blend_attach = &[
            ash::vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(ash::vk::ColorComponentFlags::R
                                  | ash::vk::ColorComponentFlags::G
                                  | ash::vk::ColorComponentFlags::B
                                  | ash::vk::ColorComponentFlags::A)
                .blend_enable(false)
                .build(),
        ];

        let blender = ash::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(blend_attach);

        let create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .color_blend_state(&blender)
            .layout(pipeline_layout)
            .render_pass(render_pass);

        unsafe {
            dev.create_graphics_pipelines(
                ash::vk::PipelineCache::null(),
                &[create_info.build()],
                None)
        }
    }.expect("Failed to create graphics pipeline")[0];

    let framebuffers: Vec<_> = present_views.iter().map(|&view| {
        let attachments =  &[view];
        let create_info = ash::vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(surface_resolution.width)
            .height(surface_resolution.height)
            .layers(1);

        unsafe { dev.create_framebuffer(&create_info, None) }.expect("Failed to create framebuffer")
    }).collect();

    let command_pool = {
        let create_info = ash::vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_idx.unwrap());
        unsafe { dev.create_command_pool(&create_info, None) }
    }.expect("Failed to create command pool");

    let command_buffers = {
        let create_info = ash::vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(ash::vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(present_views.len() as u32);
        unsafe { dev.allocate_command_buffers(&create_info) }
    }.expect("Failed to create command buffers");

    for i in 0..command_buffers.len() {
        let begin_info = ash::vk::CommandBufferBeginInfo::builder();
        unsafe { dev.begin_command_buffer(command_buffers[i], &begin_info) }
            .expect("Failed to begin recording");

        let clear_values = &[
            ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0]
                }
            }
        ];

        let render_pass_begin_info = ash::vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffers[i])
            .render_area(ash::vk::Rect2D {
                offset: ash::vk::Offset2D {
                    x:  0, y: 0,
                },

                extent: surface_resolution,
            })
            .clear_values(clear_values);

        unsafe {
            dev.cmd_begin_render_pass(
                command_buffers[i],
                &render_pass_begin_info,
                ash::vk::SubpassContents::INLINE)
        };

        unsafe {
            dev.cmd_bind_pipeline(
                command_buffers[i],
                ash::vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline
            )
        };

        unsafe {
            dev.cmd_bind_vertex_buffers(
                command_buffers[i],
                0,
                &[vertex_buffer],
                &[0]
            )
        };

        unsafe {
            dev.cmd_bind_index_buffer(
                command_buffers[i],
                index_buffer,
                0,
                ash::vk::IndexType::UINT16
            )
        };

        unsafe {
            dev.cmd_draw_indexed(
                command_buffers[i],
                /* index count */   indicies.len() as u32,
                /* instance count */ 1,
                /* first vertex */   0,
                /* vertex offset */  0,
                /* first instance */ 0,
            )
        };

        unsafe { dev.cmd_end_render_pass(command_buffers[i]) };
        unsafe { dev.end_command_buffer(command_buffers[i]) }.expect("Failed to end command buffer");
    }

    let image_ready = {
        let create_info = ash::vk::SemaphoreCreateInfo::builder();
        unsafe { dev.create_semaphore(&create_info, None) }
    }.expect("Failed to create semaphore");

    let render_done = {
        let create_info = ash::vk::SemaphoreCreateInfo::builder();
        unsafe { dev.create_semaphore(&create_info, None) }
    }.expect("Failed to create semaphore");

    // now we are ready to do stuff
    event_loop.run(move |_event, _, control_flow| {
        let next_image = unsafe { swapchain_loader.acquire_next_image(
            swapchain,
            std::u64::MAX,
            image_ready,
            ash::vk::Fence::null()
        ) }.expect("Failed to get next image"); // figure out what the bool is

        let sem_in_arr = &[image_ready];
        let sem_out_arr = &[render_done];
        let stage_arr = &[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let buf_arr = &[command_buffers[next_image.0 as usize]];

        let submit_info = ash::vk::SubmitInfo::builder()
            .wait_semaphores(sem_in_arr)
            .wait_dst_stage_mask(stage_arr)
            .command_buffers(buf_arr)
            .signal_semaphores(sem_out_arr);

        unsafe {
            dev.queue_submit(
                queue,
                &[submit_info.build()],
                ash::vk::Fence::null())
        }.expect("Failed to submit");

        let sem_in_arr = &[render_done];
        let swapchain_arr = &[swapchain];
        let idx_arr = &[next_image.0];

        let present_info = ash::vk::PresentInfoKHR::builder()
            .wait_semaphores(sem_in_arr)
            .swapchains(swapchain_arr)
            .image_indices(idx_arr);

        unsafe { swapchain_loader.queue_present(queue, &present_info) }.expect("Failed to present");
        unsafe { dev.device_wait_idle() }.expect("Failed to wait idle"); // cheap sync method

        *control_flow = winit::event_loop::ControlFlow::Poll;
    });

    // FIXME cleanup
}
