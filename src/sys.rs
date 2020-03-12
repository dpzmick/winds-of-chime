#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::os::raw::*;
use vk_functions::vk_functions;

// references
// https://vulkan.lunarg.com/doc/view/1.0.26.0/linux/vkspec.chunked/ch02s03.html
// https://github.com/KhronosGroup/Vulkan-LoaderAndValidationLayers/blob/master/loader/LoaderAndLayerInterface.md#indirectly-linking-to-the-loader

include!(concat!(env!("OUT_DIR"), "/vk_bindgen.rs"));

vk_functions!(
    struct VkInstanceBootstrap
    loader vkGetInstanceProcAddr

    fn vkCreateInstance(create_info: *const VkInstanceCreateInfo,
                        allocator:   *const VkAllocationCallbacks,
                        instance:    *mut VkInstance
    ) -> VkResult;
);

vk_functions!(
    struct VkInstancePointers
    loader vkGetInstanceProcAddr

    fn vkCreateInstance(create_info: *const VkInstanceCreateInfo,
                        allocator:   *const VkAllocationCallbacks,
                        instance:    *mut VkInstance
    ) -> VkResult;

    fn vkDestroyInstance(instance:  VkInstance,
                         allocator: *const VkAllocationCallbacks
    ) -> ();

    fn vkEnumeratePhysicalDevices(instance:       VkInstance,
                                  instance_count: *mut u32,
                                  devices:        *mut VkPhysicalDevice
    ) -> VkResult;

    fn vkGetPhysicalDeviceProperties(device:     *const VkPhysicalDevice_T,
                                     properties: *mut VkPhysicalDeviceProperties
    ) -> ();

    fn vkGetPhysicalDeviceFeatures(device: *const VkPhysicalDevice_T,
                                   features: *mut VkPhysicalDeviceFeatures
    ) -> ();

    fn vkGetPhysicalDeviceQueueFamilyProperties(device: *const VkPhysicalDevice_T,
                                                cnt: *mut u32,
                                                props: *mut VkQueueFamilyProperties
    ) -> ();

    fn vkGetPhysicalDeviceMemoryProperties(device: *const VkPhysicalDevice_T,
                                           properties: *mut VkPhysicalDeviceMemoryProperties
    ) -> ();
);
