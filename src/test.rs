use ::bitflags::bitflags;
type VkFlags = u32;
bitflags! {
    pub struct Queue: VkFlags {
        const GRAPHICS = 1<<0;
        const COMPUTE = 1<<1;
        const TRANSFER = 1<<2;
        const SPARSE_BINDING = 1<<3;
    }
}
bitflags! {
    pub struct MemoryProperty: VkFlags {
        const DEVICE_LOCAL = 1<<0;
        const HOST_VISIBLE = 1<<1;
        const HOST_COHERENT = 1<<2;
        const HOST_CACHED = 1<<3;
        const LAZILY_ALLOCATED = 1<<4;
    }
}
bitflags! {
    pub struct MemoryHeap: VkFlags {
        const DEVICE_LOCAL = 1<<0;
    }
}
bitflags! {
    pub struct Access: VkFlags {
        const INDIRECT_COMMAND_READ = 1<<0;
        const INDEX_READ = 1<<1;
        const VERTEX_ATTRIBUTE_READ = 1<<2;
        const UNIFORM_READ = 1<<3;
        const INPUT_ATTACHMENT_READ = 1<<4;
        const SHADER_READ = 1<<5;
        const SHADER_WRITE = 1<<6;
        const COLOR_ATTACHMENT_READ = 1<<7;
        const COLOR_ATTACHMENT_WRITE = 1<<8;
        const DEPTH_STENCIL_ATTACHMENT_READ = 1<<9;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 1<<10;
        const TRANSFER_READ = 1<<11;
        const TRANSFER_WRITE = 1<<12;
        const HOST_READ = 1<<13;
        const HOST_WRITE = 1<<14;
        const MEMORY_READ = 1<<15;
        const MEMORY_WRITE = 1<<16;
    }
}
bitflags! {
    pub struct BufferUsage: VkFlags {
        const TRANSFER_SRC = 1<<0;
        const TRANSFER_DST = 1<<1;
        const UNIFORM_TEXEL_BUFFER = 1<<2;
        const STORAGE_TEXEL_BUFFER = 1<<3;
        const UNIFORM_BUFFER = 1<<4;
        const STORAGE_BUFFER = 1<<5;
        const INDEX_BUFFER = 1<<6;
        const VERTEX_BUFFER = 1<<7;
        const INDIRECT_BUFFER = 1<<8;
    }
}
bitflags! {
    pub struct BufferCreate: VkFlags {
        const SPARSE_BINDING = 1<<0;
        const SPARSE_RESIDENCY = 1<<1;
        const SPARSE_ALIASED = 1<<2;
    }
}
bitflags! {
    pub struct ShaderStage: VkFlags {
        const VERTEX = 1<<0;
        const TESSELLATION_CONTROL = 1<<1;
        const TESSELLATION_EVALUATION = 1<<2;
        const GEOMETRY = 1<<3;
        const FRAGMENT = 1<<4;
        const COMPUTE = 1<<5;
        const ALL_GRAPHICS = 0x0000001F;
        const ALL = 0x7FFFFFFF;
    }
}
bitflags! {
    pub struct ImageUsage: VkFlags {
        const TRANSFER_SRC = 1<<0;
        const TRANSFER_DST = 1<<1;
        const SAMPLED = 1<<2;
        const STORAGE = 1<<3;
        const COLOR_ATTACHMENT = 1<<4;
        const DEPTH_STENCIL_ATTACHMENT = 1<<5;
        const TRANSIENT_ATTACHMENT = 1<<6;
        const INPUT_ATTACHMENT = 1<<7;
    }
}
bitflags! {
    pub struct ImageCreate: VkFlags {
        const SPARSE_BINDING = 1<<0;
        const SPARSE_RESIDENCY = 1<<1;
        const SPARSE_ALIASED = 1<<2;
        const MUTABLE_FORMAT = 1<<3;
        const CUBE_COMPATIBLE = 1<<4;
    }
}
bitflags! {
    pub struct PipelineCreate: VkFlags {
        const DISABLE_OPTIMIZATION = 1<<0;
        const ALLOW_DERIVATIVES = 1<<1;
        const DERIVATIVE = 1<<2;
    }
}
bitflags! {
    pub struct ColorComponent: VkFlags {
        const R = 1<<0;
        const G = 1<<1;
        const B = 1<<2;
        const A = 1<<3;
    }
}
bitflags! {
    pub struct FenceCreate: VkFlags {
        const SIGNALED = 1<<0;
    }
}
bitflags! {
    pub struct FormatFeature: VkFlags {
        const SAMPLED_IMAGE = 1<<0;
        const STORAGE_IMAGE = 1<<1;
        const STORAGE_IMAGE_ATOMIC = 1<<2;
        const UNIFORM_TEXEL_BUFFER = 1<<3;
        const STORAGE_TEXEL_BUFFER = 1<<4;
        const STORAGE_TEXEL_BUFFER_ATOMIC = 1<<5;
        const VERTEX_BUFFER = 1<<6;
        const COLOR_ATTACHMENT = 1<<7;
        const COLOR_ATTACHMENT_BLEND = 1<<8;
        const DEPTH_STENCIL_ATTACHMENT = 1<<9;
        const BLIT_SRC = 1<<10;
        const BLIT_DST = 1<<11;
        const SAMPLED_IMAGE_FILTER_LINEAR = 1<<12;
    }
}
bitflags! {
    pub struct QueryControl: VkFlags {
        const PRECISE = 1<<0;
    }
}
bitflags! {
    pub struct QueryResult: VkFlags {
        const V_64 = 1<<0;
        const WAIT = 1<<1;
        const WITH_AVAILABILITY = 1<<2;
        const PARTIAL = 1<<3;
    }
}
bitflags! {
    pub struct CommandPoolCreate: VkFlags {
        const TRANSIENT = 1<<0;
        const RESET_COMMAND_BUFFER = 1<<1;
    }
}
bitflags! {
    pub struct CommandPoolReset: VkFlags {
        const RELEASE_RESOURCES = 1<<0;
    }
}
bitflags! {
    pub struct CommandBufferReset: VkFlags {
        const RELEASE_RESOURCES = 1<<0;
    }
}
bitflags! {
    pub struct CommandBufferUsage: VkFlags {
        const ONE_TIME_SUBMIT = 1<<0;
        const RENDER_PASS_CONTINUE = 1<<1;
        const SIMULTANEOUS_USE = 1<<2;
    }
}
bitflags! {
    pub struct QueryPipelineStatistic: VkFlags {
        const INPUT_ASSEMBLY_VERTICES = 1<<0;
        const INPUT_ASSEMBLY_PRIMITIVES = 1<<1;
        const VERTEX_SHADER_INVOCATIONS = 1<<2;
        const GEOMETRY_SHADER_INVOCATIONS = 1<<3;
        const GEOMETRY_SHADER_PRIMITIVES = 1<<4;
        const CLIPPING_INVOCATIONS = 1<<5;
        const CLIPPING_PRIMITIVES = 1<<6;
        const FRAGMENT_SHADER_INVOCATIONS = 1<<7;
        const TESSELLATION_CONTROL_SHADER_PATCHES = 1<<8;
        const TESSELLATION_EVALUATION_SHADER_INVOCATIONS = 1<<9;
        const COMPUTE_SHADER_INVOCATIONS = 1<<10;
    }
}
bitflags! {
    pub struct ImageAspect: VkFlags {
        const COLOR = 1<<0;
        const DEPTH = 1<<1;
        const STENCIL = 1<<2;
        const METADATA = 1<<3;
    }
}
bitflags! {
    pub struct SparseMemoryBind: VkFlags {
        const METADATA = 1<<0;
    }
}
bitflags! {
    pub struct SparseImageFormat: VkFlags {
        const SINGLE_MIPTAIL = 1<<0;
        const ALIGNED_MIP_SIZE = 1<<1;
        const NONSTANDARD_BLOCK_SIZE = 1<<2;
    }
}
bitflags! {
    pub struct PipelineStage: VkFlags {
        const TOP_OF_PIPE = 1<<0;
        const DRAW_INDIRECT = 1<<1;
        const VERTEX_INPUT = 1<<2;
        const VERTEX_SHADER = 1<<3;
        const TESSELLATION_CONTROL_SHADER = 1<<4;
        const TESSELLATION_EVALUATION_SHADER = 1<<5;
        const GEOMETRY_SHADER = 1<<6;
        const FRAGMENT_SHADER = 1<<7;
        const EARLY_FRAGMENT_TESTS = 1<<8;
        const LATE_FRAGMENT_TESTS = 1<<9;
        const COLOR_ATTACHMENT_OUTPUT = 1<<10;
        const COMPUTE_SHADER = 1<<11;
        const TRANSFER = 1<<12;
        const BOTTOM_OF_PIPE = 1<<13;
        const HOST = 1<<14;
        const ALL_GRAPHICS = 1<<15;
        const ALL_COMMANDS = 1<<16;
    }
}
bitflags! {
    pub struct SampleCount: VkFlags {
        const V_1 = 1<<0;
        const V_2 = 1<<1;
        const V_4 = 1<<2;
        const V_8 = 1<<3;
        const V_16 = 1<<4;
        const V_32 = 1<<5;
        const V_64 = 1<<6;
    }
}
bitflags! {
    pub struct AttachmentDescription: VkFlags {
        const MAY_ALIAS = 1<<0;
    }
}
bitflags! {
    pub struct StencilFace: VkFlags {
        const FRONT = 1<<0;
        const BACK = 1<<1;
        const FRONT_AND_BACK = 0x00000003;
    }
}
bitflags! {
    pub struct CullMode: VkFlags {
        const NONE = 0;
        const FRONT = 1<<0;
        const BACK = 1<<1;
        const FRONT_AND_BACK = 0x00000003;
    }
}
bitflags! {
    pub struct DescriptorPoolCreate: VkFlags {
        const FREE_DESCRIPTOR_SET = 1<<0;
    }
}
bitflags! {
    pub struct Dependency: VkFlags {
        const BY_REGION = 1<<0;
    }
}
bitflags! {
    pub struct SubgroupFeature: VkFlags {
        const BASIC = 1<<0;
        const VOTE = 1<<1;
        const ARITHMETIC = 1<<2;
        const BALLOT = 1<<3;
        const SHUFFLE = 1<<4;
        const SHUFFLE_RELATIVE = 1<<5;
        const CLUSTERED = 1<<6;
        const QUAD = 1<<7;
    }
}
bitflags! {
    pub struct IndirectCommandsLayoutUsageNVX: VkFlags {
        const UNORDERED_SEQUENCES = 1<<0;
        const SPARSE_SEQUENCES = 1<<1;
        const EMPTY_EXECUTIONS = 1<<2;
        const INDEXED_SEQUENCES = 1<<3;
    }
}
bitflags! {
    pub struct ObjectEntryUsageNVX: VkFlags {
        const GRAPHICS = 1<<0;
        const COMPUTE = 1<<1;
    }
}
bitflags! {
    pub struct GeometryNV: VkFlags {
        const OPAQUE = 1<<0;
        const NO_DUPLICATE_ANY_HIT_INVOCATION = 1<<1;
    }
}
bitflags! {
    pub struct GeometryInstanceNV: VkFlags {
        const TRIANGLE_CULL_DISABLE = 1<<0;
        const TRIANGLE_FRONT_COUNTERCLOCKWISE = 1<<1;
        const FORCE_OPAQUE = 1<<2;
        const FORCE_NO_OPAQUE = 1<<3;
    }
}
bitflags! {
    pub struct BuildAccelerationStructureNV: VkFlags {
        const ALLOW_UPDATE = 1<<0;
        const ALLOW_COMPACTION = 1<<1;
        const PREFER_FAST_TRACE = 1<<2;
        const PREFER_FAST_BUILD = 1<<3;
        const LOW_MEMORY = 1<<4;
    }
}
bitflags! {
    pub struct PipelineCreationFeedbackEXT: VkFlags {
        const VALID = 1<<0;
        const APPLICATION_PIPELINE_CACHE_HIT = 1<<1;
        const BASE_PIPELINE_ACCELERATION = 1<<2;
    }
}
bitflags! {
    pub struct PerformanceCounterDescriptionKHR: VkFlags {
        const PERFORMANCE_IMPACTING_KHR = 1<<0;
        const CONCURRENTLY_IMPACTED_KHR = 1<<1;
    }
}
bitflags! {
    pub struct SemaphoreWait: VkFlags {
        const ANY = 1<<0;
    }
}
bitflags! {
    pub struct CompositeAlphaKHR: VkFlags {
        const OPAQUE = 1<<0;
        const PRE_MULTIPLIED = 1<<1;
        const POST_MULTIPLIED = 1<<2;
        const INHERIT = 1<<3;
    }
}
bitflags! {
    pub struct DisplayPlaneAlphaKHR: VkFlags {
        const OPAQUE = 1<<0;
        const GLOBAL = 1<<1;
        const PER_PIXEL = 1<<2;
        const PER_PIXEL_PREMULTIPLIED = 1<<3;
    }
}
bitflags! {
    pub struct SurfaceTransformKHR: VkFlags {
        const IDENTITY = 1<<0;
        const ROTATE_90 = 1<<1;
        const ROTATE_180 = 1<<2;
        const ROTATE_270 = 1<<3;
        const HORIZONTAL_MIRROR = 1<<4;
        const HORIZONTAL_MIRROR_ROTATE_90 = 1<<5;
        const HORIZONTAL_MIRROR_ROTATE_180 = 1<<6;
        const HORIZONTAL_MIRROR_ROTATE_270 = 1<<7;
        const INHERIT = 1<<8;
    }
}
bitflags! {
    pub struct PeerMemoryFeature: VkFlags {
        const COPY_SRC = 1<<0;
        const COPY_DST = 1<<1;
        const GENERIC_SRC = 1<<2;
        const GENERIC_DST = 1<<3;
    }
}
bitflags! {
    pub struct MemoryAllocate: VkFlags {
        const DEVICE_MASK = 1<<0;
    }
}
bitflags! {
    pub struct DeviceGroupPresentModeKHR: VkFlags {
        const LOCAL = 1<<0;
        const REMOTE = 1<<1;
        const SUM = 1<<2;
        const LOCAL_MULTI_DEVICE = 1<<3;
    }
}
bitflags! {
    pub struct DebugReportEXT: VkFlags {
        const INFORMATION = 1<<0;
        const WARNING = 1<<1;
        const PERFORMANCE_WARNING = 1<<2;
        const ERROR = 1<<3;
        const DEBUG = 1<<4;
    }
}
bitflags! {
    pub struct ExternalMemoryHandleTypeNV: VkFlags {
        const OPAQUE_WIN32 = 1<<0;
        const OPAQUE_WIN32_KMT = 1<<1;
        const D3D11_IMAGE = 1<<2;
        const D3D11_IMAGE_KMT = 1<<3;
    }
}
bitflags! {
    pub struct ExternalMemoryFeatureNV: VkFlags {
        const DEDICATED_ONLY = 1<<0;
        const EXPORTABLE = 1<<1;
        const IMPORTABLE = 1<<2;
    }
}
bitflags! {
    pub struct ExternalMemoryHandleType: VkFlags {
        const OPAQUE_FD = 1<<0;
        const OPAQUE_WIN32 = 1<<1;
        const OPAQUE_WIN32_KMT = 1<<2;
        const D3D11_TEXTURE = 1<<3;
        const D3D11_TEXTURE_KMT = 1<<4;
        const D3D12_HEAP = 1<<5;
        const D3D12_RESOURCE = 1<<6;
    }
}
bitflags! {
    pub struct ExternalMemoryFeature: VkFlags {
        const DEDICATED_ONLY = 1<<0;
        const EXPORTABLE = 1<<1;
        const IMPORTABLE = 1<<2;
    }
}
bitflags! {
    pub struct ExternalSemaphoreHandleType: VkFlags {
        const OPAQUE_FD = 1<<0;
        const OPAQUE_WIN32 = 1<<1;
        const OPAQUE_WIN32_KMT = 1<<2;
        const D3D12_FENCE = 1<<3;
        const SYNC_FD = 1<<4;
    }
}
bitflags! {
    pub struct ExternalSemaphoreFeature: VkFlags {
        const EXPORTABLE = 1<<0;
        const IMPORTABLE = 1<<1;
    }
}
bitflags! {
    pub struct SemaphoreImport: VkFlags {
        const TEMPORARY = 1<<0;
    }
}
bitflags! {
    pub struct ExternalFenceHandleType: VkFlags {
        const OPAQUE_FD = 1<<0;
        const OPAQUE_WIN32 = 1<<1;
        const OPAQUE_WIN32_KMT = 1<<2;
        const SYNC_FD = 1<<3;
    }
}
bitflags! {
    pub struct ExternalFenceFeature: VkFlags {
        const EXPORTABLE = 1<<0;
        const IMPORTABLE = 1<<1;
    }
}
bitflags! {
    pub struct FenceImport: VkFlags {
        const TEMPORARY = 1<<0;
    }
}
bitflags! {
    pub struct SurfaceCounterEXT: VkFlags {
        const VBLANK_EXT = 1<<0;
    }
}
bitflags! {
    pub struct DebugUtilsMessageSeverityEXT: VkFlags {
        const VERBOSE = 1<<0;
        const INFO = 1<<4;
        const WARNING = 1<<8;
        const ERROR = 1<<12;
    }
}
bitflags! {
    pub struct DebugUtilsMessageTypeEXT: VkFlags {
        const GENERAL = 1<<0;
        const VALIDATION = 1<<1;
        const PERFORMANCE = 1<<2;
    }
}
bitflags! {
    pub struct DescriptorBinding: VkFlags {
        const UPDATE_AFTER_BIND = 1<<0;
        const UPDATE_UNUSED_WHILE_PENDING = 1<<1;
        const PARTIALLY_BOUND = 1<<2;
        const VARIABLE_DESCRIPTOR_COUNT = 1<<3;
    }
}
bitflags! {
    pub struct ConditionalRenderingEXT: VkFlags {
        const INVERTED = 1<<0;
    }
}
bitflags! {
    pub struct ResolveMode: VkFlags {
        const NONE = 0;
        const SAMPLE_ZERO = 1<<0;
        const AVERAGE = 1<<1;
        const MIN = 1<<2;
        const MAX = 1<<3;
    }
}
bitflags! {
    pub struct SwapchainImageUsageANDROID: VkFlags {
        const SHARED = 1<<0;
    }
}
bitflags! {
    pub struct ToolPurposeEXT: VkFlags {
        const VALIDATION = 1<<0;
        const PROFILING = 1<<1;
        const TRACING = 1<<2;
        const ADDITIONAL_FEATURES = 1<<3;
        const MODIFYING_FEATURES = 1<<4;
    }
}
