#![allow(dead_code)]

use crate::dynamic_library::DynamicLibrary;
use crate::sys;
use ffi::StrRef;
use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::{c_void, c_char};
use std::ptr::{null, null_mut};

#[repr(transparent)]
pub struct ApplicationInfo<'a, 'b>(
    sys::VkApplicationInfo,
    PhantomData<&'a CStr>,  // app_name
    PhantomData<&'b CStr>,  // engine_name
);

impl<'a, 'b> ApplicationInfo<'a, 'b> {
    fn new_empty() -> Self {
        let mut zeroed = unsafe { mem::zeroed::<sys::VkApplicationInfo>() };
        zeroed.sType = sys::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO;
        Self(
            zeroed,
            PhantomData,
            PhantomData,
        )
    }
}

pub struct ApplicationInfoBuilder<'a, 'b> {
    inner: ApplicationInfo<'a, 'b>,
}

impl<'a, 'b> ApplicationInfoBuilder<'a, 'b> {
    pub fn new() -> Self {
        Self {
            inner: ApplicationInfo::new_empty(),
        }
    }

    pub fn application_name(mut self, name: &'a CStr) -> Self {
        self.inner.0.pApplicationName = name.as_ptr();
        self
    }

    pub fn application_version(mut self, version: u32) -> Self {
        self.inner.0.applicationVersion = version;
        self
    }

    pub fn engine_name(mut self, name: &'b CStr) -> Self {
        self.inner.0.pEngineName = name.as_ptr();
        self
    }

    pub fn engine_version(mut self, version: u32) -> Self {
        self.inner.0.engineVersion = version;
        self
    }

    pub fn build(self) -> ApplicationInfo<'a, 'b> {
        self.inner
    }
}

#[repr(transparent)]
pub struct InstanceCreateInfo<'a, 'b, 'c, 'd, 'e>(
    sys::VkInstanceCreateInfo,
    PhantomData<&'a CStr>,       // app_info
    PhantomData<&'b [&'c CStr]>, // layer_names
    PhantomData<&'d [&'e CStr]>, // enabled_extension_names
);

impl<'a, 'b, 'c, 'd, 'e> InstanceCreateInfo<'a, 'b, 'c, 'd, 'e> {
    fn new_empty() -> Self {
        let mut zeroed = unsafe { mem::zeroed::<sys::VkInstanceCreateInfo>() };
        zeroed.sType = sys::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
        Self(
            zeroed,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

pub struct InstanceCreateInfoBuilder<'a, 'b, 'c, 'd, 'e> {
    inner: InstanceCreateInfo<'a, 'b, 'c, 'd, 'e>,
}

impl<'a, 'b, 'c, 'd, 'e> InstanceCreateInfoBuilder<'a, 'b, 'c, 'd, 'e> {
    pub fn new() -> Self {
        Self {
            inner: InstanceCreateInfo::new_empty()
        }
    }

    pub fn application_info(mut self, app_info: &'a ApplicationInfo) -> Self {
        self.inner.0.pApplicationInfo = &app_info.0;
        self
    }

    // allow strings to live for shorter time
    pub fn enabled_layers(mut self, layers: &'b [StrRef<'c>]) -> Self {
        self.inner.0.enabledLayerCount = layers.len() as u32;
        self.inner.0.ppEnabledLayerNames = layers.as_ptr() as *const *const c_char;
        self
    }

    pub fn enabled_extensions(mut self, exts: &'d [StrRef<'e>]) -> Self {
        self.inner.0.enabledExtensionCount = exts.len() as u32;
        self.inner.0.ppEnabledExtensionNames = exts.as_ptr() as *const *const c_char;
        self
    }

    pub fn build(self) -> InstanceCreateInfo<'a, 'b, 'c, 'd, 'e> {
        self.inner
    }
}

// Cannot be transparent, we need to hold onto the function pointers!
pub struct Instance<'pointers> {
    inner: sys::VkInstance,         // this is a pointer
    ptrs:  sys::VkInstancePointers,

    // implicit lifetime on the function pointers
    ptr_lifetime: PhantomData<&'pointers DynamicLibrary>,
}

impl<'pointers> Instance<'pointers> {
    #[inline(never)]
    pub fn new(library: &DynamicLibrary, create_info: &InstanceCreateInfo) -> Self {
        // two step loader, first create the instance the load the
        // bulk of the pointers

        let bootstrap = sys::VkInstanceBootstrap::load(|nm| {
            unsafe { library.sym(nm).unwrap() }
        });

        let mut instance = null_mut();
        let result = unsafe {
            (bootstrap.vkCreateInstance)(
                &create_info.0,
                null(),
                &mut instance)
        };

        // FIXME error codes
        if instance.is_null() {
            panic!("failed to create {:?}", result);
        }

        let ptrs = sys::VkInstancePointers::load_with_arg(instance as *const c_void, |nm|{
            unsafe { library.sym(nm).unwrap() }
        });

        Self {
            inner: instance,
            ptrs: ptrs,
            ptr_lifetime: PhantomData,
        }
    }

    #[inline(never)]
    pub fn enumerate_physical_devices(&self) -> Vec<&PhysicalDevice> {
        let f = self.ptrs.vkEnumeratePhysicalDevices;
        unsafe {
            // FIXME error codes
            let mut cnt: u32 = 0;
            let err = f(self.inner, &mut cnt, null_mut());

            let mut raw = Vec::with_capacity(cnt as usize);
            let err = f(self.inner, &mut cnt, raw.as_mut_ptr());
            raw.set_len(cnt as usize);

            mem::transmute( raw ) // careful, can't do this without transmute
        }

        // decided not to box the device, instead require all the
        // calls about the physical device to be made to the instance.
    }

    #[inline(never)]
    pub fn get_physical_device_properties(&self, physical_device: &PhysicalDevice)
        -> PhysicalDeviceProperties
    {
        unsafe {
            let mut ret = mem::MaybeUninit::uninit();
            // FIXME error codes
            (self.ptrs.vkGetPhysicalDeviceProperties)(
                physical_device,
                ret.as_mut_ptr());

            ret.assume_init()
        }
    }

    #[inline(never)]
    pub fn get_physical_device_features(&self, physical_device: &PhysicalDevice)
        -> PhysicalDeviceFeatures
    {
        unsafe {
            let mut ret = mem::MaybeUninit::uninit();
            (self.ptrs.vkGetPhysicalDeviceFeatures)(
                physical_device,
                ret.as_mut_ptr()
            );

            ret.assume_init()
        }
    }

    #[inline(never)]
    pub fn get_physical_device_queue_family_properties(&self, physical_device: &PhysicalDevice)
        -> Vec<QueueFamilyProperties>
    {
        // FIXME error codes
        unsafe {
            let mut cnt: u32 = 0;
            (self.ptrs.vkGetPhysicalDeviceQueueFamilyProperties)(
                physical_device,
                &mut cnt,
                null_mut()
            );

            let mut ret = Vec::with_capacity(cnt as usize);
            (self.ptrs.vkGetPhysicalDeviceQueueFamilyProperties)(
                physical_device,
                &mut cnt,
                ret.as_mut_ptr()
            );

            ret.set_len(cnt as usize);
            ret
        }
    }

    #[inline(never)]
    pub fn get_physical_device_memory_properties(&self, physical_device: &PhysicalDevice)
        -> PhysicalDeviceMemoryProperties
    {
        unsafe {
            let mut ret = mem::MaybeUninit::uninit();
            (self.ptrs.vkGetPhysicalDeviceMemoryProperties)(
                physical_device,
                ret.as_mut_ptr()
            );

            ret.assume_init()
        }
    }
}

impl<'pointers> Drop for Instance<'pointers> {
    #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            (self.ptrs.vkDestroyInstance)(self.inner, null())
        }
    }
}

type PhysicalDevice           = sys::VkPhysicalDevice_T;
type PhysicalDeviceProperties = sys::VkPhysicalDeviceProperties;

impl PhysicalDeviceProperties {
    pub fn api_version(&self) -> u32                { self.apiVersion }
    pub fn driver_version(&self) -> u32             { self.driverVersion }
    pub fn vendor_id(&self) -> u32                  { self.vendorID }
    pub fn device_id(&self) -> u32                  { self.deviceID }
    pub fn device_type(&self) -> PhysicalDeviceType { self.deviceType }

    pub fn device_name(&self) -> &str {
        unsafe {
            // this is safe, slice/cstr are fat pointer (size,ptr)
            let cstr = CStr::from_ptr(self.deviceName.as_ptr());
            match cstr.to_str() {
                Ok(ret) => ret,
                Err(_)  => unreachable!(), // better codegen, probably unlikely
            }
        }
    }

    pub fn pipeline_cache_uuid(&self) -> [u8; 16]                        { self.pipelineCacheUUID }
    pub fn limits(&self)              -> &PhysicalDeviceLimits           { &self.limits }
    pub fn sparse_properties(&self)   -> &PhysicalDeviceSparseProperties { &self.sparseProperties }
}

// impl fmt::Debug for PhysicalDeviceProperties
// need to format CStr manually

type PhysicalDeviceType             = sys::VkPhysicalDeviceType;
type PhysicalDeviceLimits           = sys::VkPhysicalDeviceLimits;
type PhysicalDeviceSparseProperties = sys::VkPhysicalDeviceSparseProperties;
type PhysicalDeviceFeatures         = sys::VkPhysicalDeviceFeatures;
type PhysicalDeviceMemoryProperties = sys::VkPhysicalDeviceMemoryProperties;

impl PhysicalDeviceMemoryProperties {
    #[inline(never)]
    #[cfg(debug_assertions)]
    pub fn memory_types(&self) -> &[MemoryType] {
        &self.memoryTypes[0..self.memoryTypeCount as usize]
    }

    #[inline(never)]
    #[cfg(not(debug_assertions))]
    pub fn memory_types(&self) -> &[MemoryType] {
        unsafe {
            std::slice::from_raw_parts(
                self.memoryTypes.as_ptr(),
                self.memoryTypeCount as usize)
        }
    }

    #[inline(never)]
    #[cfg(debug_assertions)]
    pub fn memory_heaps(&self) -> &[MemoryHeap] {
        &self.memoryHeaps[0..self.memoryHeapCount as usize]
    }

    #[inline(never)]
    #[cfg(not(debug_assertions))]
    pub fn memory_heaps(&self) -> &[MemoryHeap] {
        unsafe {
            std::slice::from_raw_parts(
                self.memoryHeaps.as_ptr(),
                self.memoryHeapCount as usize)
        }
    }
}

// queue types
type QueueFamilyProperties = sys::VkQueueFamilyProperties;
type QueueFlags            = sys::VkQueueFlagsBits;

impl fmt::Display for QueueFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueueFlags: ")?;

        let mut bits = Vec::new(); // FIXME
        if self & Self::VK_QUEUE_GRAPHICS_BIT {
            bits.push("Graphics");
        }

        if self & Self::VK_QUEUE_COMPUTE_BIT {
            bits.push("Compute");
        }

        if self | Self::VK_QUEUE_TRANSFER_BIT {
            bits.push("Transfer");
        }

        if self & Self::VK_SPARSE_BINDING_BIT {
            bits.push("SparseBinding");
        }

        if self & Self::VK_QUEUE_PROTECTED_BIT {
            bits.push("Protected");
        }

    }
}

type MemoryType                     = sys::VkMemoryType;
type MemoryHeap                     = sys::VkMemoryHeap;


// FIXME remove the inline nevers for setup code
// or keep?
