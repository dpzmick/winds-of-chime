use bitflags::bitflags;
bitflags! {
    struct CullMode : u32 {
          const NONE = 0;
          const FRONT = 0x1;
          const BACK = 0x2;
          const FRONT_AND_BACK = 0x00000003;
    }
}
bitflags! {
    struct Queue : u32 {
          const GRAPHICS = 0x1;
          const COMPUTE = 0x2;
          const TRANSFER = 0x4;
          const SPARSE_BINDING = 0x8;
    }
}
bitflags! {
    struct MemoryProperty : u32 {
          const DEVICE_LOCAL = 0x1;
          const HOST_VISIBLE = 0x2;
          const HOST_COHERENT = 0x4;
          const HOST_CACHED = 0x8;
          const LAZILY_ALLOCATED = 0x10;
    }
}
bitflags! {
    struct MemoryHeap : u32 {
          const DEVICE_LOCAL = 0x1;
    }
}
bitflags! {
    struct Access : u32 {
          const INDIRECT_COMMAND_READ = 0x1;
          const INDEX_READ = 0x2;
          const VERTEX_ATTRIBUTE_READ = 0x4;
          const UNIFORM_READ = 0x8;
          const INPUT_ATTACHMENT_READ = 0x10;
          const SHADER_READ = 0x20;
          const SHADER_WRITE = 0x40;
          const COLOR_ATTACHMENT_READ = 0x80;
          const COLOR_ATTACHMENT_WRITE = 0x100;
          const DEPTH_STENCIL_ATTACHMENT_READ = 0x200;
          const DEPTH_STENCIL_ATTACHMENT_WRITE = 0x400;
          const TRANSFER_READ = 0x800;
          const TRANSFER_WRITE = 0x1000;
          const HOST_READ = 0x2000;
          const HOST_WRITE = 0x4000;
          const MEMORY_READ = 0x8000;
          const MEMORY_WRITE = 0x10000;
    }
}
bitflags! {
    struct BufferUsage : u32 {
          const TRANSFER_SRC = 0x1;
          const TRANSFER_DST = 0x2;
          const UNIFORM_TEXEL_BUFFER = 0x4;
          const STORAGE_TEXEL_BUFFER = 0x8;
          const UNIFORM_BUFFER = 0x10;
          const STORAGE_BUFFER = 0x20;
          const INDEX_BUFFER = 0x40;
          const VERTEX_BUFFER = 0x80;
          const INDIRECT_BUFFER = 0x100;
    }
}
bitflags! {
    struct BufferCreate : u32 {
          const SPARSE_BINDING = 0x1;
          const SPARSE_RESIDENCY = 0x2;
          const SPARSE_ALIASED = 0x4;
    }
}
bitflags! {
    struct ShaderStage : u32 {
          const VERTEX = 0x1;
          const TESSELLATION_CONTROL = 0x2;
          const TESSELLATION_EVALUATION = 0x4;
          const GEOMETRY = 0x8;
          const FRAGMENT = 0x10;
          const COMPUTE = 0x20;
          const ALL_GRAPHICS = 0x0000001F;
          const ALL = 0x7FFFFFFF;
    }
}
bitflags! {
    struct ImageUsage : u32 {
          const TRANSFER_SRC = 0x1;
          const TRANSFER_DST = 0x2;
          const SAMPLED = 0x4;
          const STORAGE = 0x8;
          const COLOR_ATTACHMENT = 0x10;
          const DEPTH_STENCIL_ATTACHMENT = 0x20;
          const TRANSIENT_ATTACHMENT = 0x40;
          const INPUT_ATTACHMENT = 0x80;
    }
}
bitflags! {
    struct ImageCreate : u32 {
          const SPARSE_BINDING = 0x1;
          const SPARSE_RESIDENCY = 0x2;
          const SPARSE_ALIASED = 0x4;
          const MUTABLE_FORMAT = 0x8;
          const CUBE_COMPATIBLE = 0x10;
    }
}
bitflags! {
    struct PipelineCreate : u32 {
          const DISABLE_OPTIMIZATION = 0x1;
          const ALLOW_DERIVATIVES = 0x2;
          const DERIVATIVE = 0x4;
    }
}
bitflags! {
    struct ColorComponent : u32 {
          const R = 0x1;
          const G = 0x2;
          const B = 0x4;
          const A = 0x8;
    }
}
bitflags! {
    struct FenceCreate : u32 {
          const SIGNALED = 0x1;
    }
}
bitflags! {
    struct FormatFeature : u32 {
          const SAMPLED_IMAGE = 0x1;
          const STORAGE_IMAGE = 0x2;
          const STORAGE_IMAGE_ATOMIC = 0x4;
          const UNIFORM_TEXEL_BUFFER = 0x8;
          const STORAGE_TEXEL_BUFFER = 0x10;
          const STORAGE_TEXEL_BUFFER_ATOMIC = 0x20;
          const VERTEX_BUFFER = 0x40;
          const COLOR_ATTACHMENT = 0x80;
          const COLOR_ATTACHMENT_BLEND = 0x100;
          const DEPTH_STENCIL_ATTACHMENT = 0x200;
          const BLIT_SRC = 0x400;
          const BLIT_DST = 0x800;
          const SAMPLED_IMAGE_FILTER_LINEAR = 0x1000;
    }
}
bitflags! {
    struct QueryControl : u32 {
          const PRECISE = 0x1;
    }
}
bitflags! {
    struct QueryResult : u32 {
          const VALUE_64 = 0x1;
          const WAIT = 0x2;
          const WITH_AVAILABILITY = 0x4;
          const PARTIAL = 0x8;
    }
}
bitflags! {
    struct CommandBufferUsage : u32 {
          const ONE_TIME_SUBMIT = 0x1;
          const RENDER_PASS_CONTINUE = 0x2;
          const SIMULTANEOUS_USE = 0x4;
    }
}
bitflags! {
    struct QueryPipelineStatistic : u32 {
          const INPUT_ASSEMBLY_VERTICES = 0x1;
          const INPUT_ASSEMBLY_PRIMITIVES = 0x2;
          const VERTEX_SHADER_INVOCATIONS = 0x4;
          const GEOMETRY_SHADER_INVOCATIONS = 0x8;
          const GEOMETRY_SHADER_PRIMITIVES = 0x10;
          const CLIPPING_INVOCATIONS = 0x20;
          const CLIPPING_PRIMITIVES = 0x40;
          const FRAGMENT_SHADER_INVOCATIONS = 0x80;
          const TESSELLATION_CONTROL_SHADER_PATCHES = 0x100;
          const TESSELLATION_EVALUATION_SHADER_INVOCATIONS = 0x200;
          const COMPUTE_SHADER_INVOCATIONS = 0x400;
    }
}
bitflags! {
    struct ImageAspect : u32 {
          const COLOR = 0x1;
          const DEPTH = 0x2;
          const STENCIL = 0x4;
          const METADATA = 0x8;
    }
}
bitflags! {
    struct SparseImageFormat : u32 {
          const SINGLE_MIPTAIL = 0x1;
          const ALIGNED_MIP_SIZE = 0x2;
          const NONSTANDARD_BLOCK_SIZE = 0x4;
    }
}
bitflags! {
    struct SparseMemoryBind : u32 {
          const METADATA = 0x1;
    }
}
bitflags! {
    struct PipelineStage : u32 {
          const TOP_OF_PIPE = 0x1;
          const DRAW_INDIRECT = 0x2;
          const VERTEX_INPUT = 0x4;
          const VERTEX_SHADER = 0x8;
          const TESSELLATION_CONTROL_SHADER = 0x10;
          const TESSELLATION_EVALUATION_SHADER = 0x20;
          const GEOMETRY_SHADER = 0x40;
          const FRAGMENT_SHADER = 0x80;
          const EARLY_FRAGMENT_TESTS = 0x100;
          const LATE_FRAGMENT_TESTS = 0x200;
          const COLOR_ATTACHMENT_OUTPUT = 0x400;
          const COMPUTE_SHADER = 0x800;
          const TRANSFER = 0x1000;
          const BOTTOM_OF_PIPE = 0x2000;
          const HOST = 0x4000;
          const ALL_GRAPHICS = 0x8000;
          const ALL_COMMANDS = 0x10000;
    }
}
bitflags! {
    struct CommandPoolCreate : u32 {
          const TRANSIENT = 0x1;
          const RESET_COMMAND_BUFFER = 0x2;
    }
}
bitflags! {
    struct CommandPoolReset : u32 {
          const RELEASE_RESOURCES = 0x1;
    }
}
bitflags! {
    struct CommandBufferReset : u32 {
          const RELEASE_RESOURCES = 0x1;
    }
}
bitflags! {
    struct SampleCount : u32 {
          const VALUE_1 = 0x1;
          const VALUE_2 = 0x2;
          const VALUE_4 = 0x4;
          const VALUE_8 = 0x8;
          const VALUE_16 = 0x10;
          const VALUE_32 = 0x20;
          const VALUE_64 = 0x40;
    }
}
bitflags! {
    struct AttachmentDescription : u32 {
          const MAY_ALIAS = 0x1;
    }
}
bitflags! {
    struct StencilFace : u32 {
          const FRONT = 0x1;
          const BACK = 0x2;
          const FRONT_AND_BACK = 0x00000003;
    }
}
bitflags! {
    struct DescriptorPoolCreate : u32 {
          const FREE_DESCRIPTOR_SET = 0x1;
    }
}
bitflags! {
    struct Dependency : u32 {
          const BY_REGION = 0x1;
    }
}
bitflags! {
    struct SemaphoreWait : u32 {
          const ANY = 0x1;
    }
}
bitflags! {
    struct DisplayPlaneAlphaKHR : u32 {
          const OPAQUE = 0x1;
          const GLOBAL = 0x2;
          const PER_PIXEL = 0x4;
          const PER_PIXEL_PREMULTIPLIED = 0x8;
    }
}
bitflags! {
    struct CompositeAlphaKHR : u32 {
          const OPAQUE = 0x1;
          const PRE_MULTIPLIED = 0x2;
          const POST_MULTIPLIED = 0x4;
          const INHERIT = 0x8;
    }
}
bitflags! {
    struct SurfaceTransformKHR : u32 {
          const IDENTITY = 0x1;
          const ROTATE_90 = 0x2;
          const ROTATE_180 = 0x4;
          const ROTATE_270 = 0x8;
          const HORIZONTAL_MIRROR = 0x10;
          const HORIZONTAL_MIRROR_ROTATE_90 = 0x20;
          const HORIZONTAL_MIRROR_ROTATE_180 = 0x40;
          const HORIZONTAL_MIRROR_ROTATE_270 = 0x80;
          const INHERIT = 0x100;
    }
}
bitflags! {
    struct SwapchainImageUsageANDROID : u32 {
          const SHARED = 0x1;
    }
}
bitflags! {
    struct DebugReportEXT : u32 {
          const INFORMATION = 0x1;
          const WARNING = 0x2;
          const PERFORMANCE_WARNING = 0x4;
          const ERROR = 0x8;
          const DEBUG = 0x10;
    }
}
bitflags! {
    struct ExternalMemoryHandleTypeNV : u32 {
          const OPAQUE_WIN32 = 0x1;
          const OPAQUE_WIN32_KMT = 0x2;
          const D3D11_IMAGE = 0x4;
          const D3D11_IMAGE_KMT = 0x8;
    }
}
bitflags! {
    struct ExternalMemoryFeatureNV : u32 {
          const DEDICATED_ONLY = 0x1;
          const EXPORTABLE = 0x2;
          const IMPORTABLE = 0x4;
    }
}
bitflags! {
    struct SubgroupFeature : u32 {
          const BASIC = 0x1;
          const VOTE = 0x2;
          const ARITHMETIC = 0x4;
          const BALLOT = 0x8;
          const SHUFFLE = 0x10;
          const SHUFFLE_RELATIVE = 0x20;
          const CLUSTERED = 0x40;
          const QUAD = 0x80;
    }
}
bitflags! {
    struct IndirectCommandsLayoutUsageNVX : u32 {
          const UNORDERED_SEQUENCES = 0x1;
          const SPARSE_SEQUENCES = 0x2;
          const EMPTY_EXECUTIONS = 0x4;
          const INDEXED_SEQUENCES = 0x8;
    }
}
bitflags! {
    struct ObjectEntryUsageNVX : u32 {
          const GRAPHICS = 0x1;
          const COMPUTE = 0x2;
    }
}
bitflags! {
    struct ExternalMemoryHandleType : u32 {
          const OPAQUE_FD = 0x1;
          const OPAQUE_WIN32 = 0x2;
          const OPAQUE_WIN32_KMT = 0x4;
          const D3D11_TEXTURE = 0x8;
          const D3D11_TEXTURE_KMT = 0x10;
          const D3D12_HEAP = 0x20;
          const D3D12_RESOURCE = 0x40;
    }
}
bitflags! {
    struct ExternalMemoryFeature : u32 {
          const DEDICATED_ONLY = 0x1;
          const EXPORTABLE = 0x2;
          const IMPORTABLE = 0x4;
    }
}
bitflags! {
    struct ExternalSemaphoreHandleType : u32 {
          const OPAQUE_FD = 0x1;
          const OPAQUE_WIN32 = 0x2;
          const OPAQUE_WIN32_KMT = 0x4;
          const D3D12_FENCE = 0x8;
          const SYNC_FD = 0x10;
    }
}
bitflags! {
    struct ExternalSemaphoreFeature : u32 {
          const EXPORTABLE = 0x1;
          const IMPORTABLE = 0x2;
    }
}
bitflags! {
    struct SemaphoreImport : u32 {
          const TEMPORARY = 0x1;
    }
}
bitflags! {
    struct ExternalFenceHandleType : u32 {
          const OPAQUE_FD = 0x1;
          const OPAQUE_WIN32 = 0x2;
          const OPAQUE_WIN32_KMT = 0x4;
          const SYNC_FD = 0x8;
    }
}
bitflags! {
    struct ExternalFenceFeature : u32 {
          const EXPORTABLE = 0x1;
          const IMPORTABLE = 0x2;
    }
}
bitflags! {
    struct FenceImport : u32 {
          const TEMPORARY = 0x1;
    }
}
bitflags! {
    struct SurfaceCounterEXT : u32 {
          const VB = 0x1;
    }
}
bitflags! {
    struct PeerMemoryFeature : u32 {
          const COPY_SRC = 0x1;
          const COPY_DST = 0x2;
          const GENERIC_SRC = 0x4;
          const GENERIC_DST = 0x8;
    }
}
bitflags! {
    struct MemoryAllocate : u32 {
          const DEVICE_MASK = 0x1;
    }
}
bitflags! {
    struct DeviceGroupPresentModeKHR : u32 {
          const LOCAL = 0x1;
          const REMOTE = 0x2;
          const SUM = 0x4;
          const LOCAL_MULTI_DEVICE = 0x8;
    }
}
bitflags! {
    struct DebugUtilsMessageSeverityEXT : u32 {
          const VERBOSE = 0x1;
          const INFO = 0x10;
          const WARNING = 0x100;
          const ERROR = 0x1000;
    }
}
bitflags! {
    struct DebugUtilsMessageTypeEXT : u32 {
          const GENERAL = 0x1;
          const VALIDATION = 0x2;
          const PERFORMANCE = 0x4;
    }
}
bitflags! {
    struct DescriptorBinding : u32 {
          const UPDATE_AFTER_BIND = 0x1;
          const UPDATE_UNUSED_WHILE_PENDING = 0x2;
          const PARTIALLY_BOUND = 0x4;
          const VARIABLE_DESCRIPTOR_COUNT = 0x8;
    }
}
bitflags! {
    struct ConditionalRenderingEXT : u32 {
          const INVERTED = 0x1;
    }
}
bitflags! {
    struct ResolveMode : u32 {
          const NONE = 0;
          const SAMPLE_ZERO = 0x1;
          const AVERAGE = 0x2;
          const MIN = 0x4;
          const MAX = 0x8;
    }
}
bitflags! {
    struct GeometryInstanceNV : u32 {
          const TRIANGLE_CULL_DISABLE = 0x1;
          const TRIANGLE_FRONT_COUNTERCLOCKWISE = 0x2;
          const FORCE_OPAQUE = 0x4;
          const FORCE_NO_OPAQUE = 0x8;
    }
}
bitflags! {
    struct GeometryNV : u32 {
          const OPAQUE = 0x1;
          const NO_DUPLICATE_ANY_HIT_INVOCATION = 0x2;
    }
}
bitflags! {
    struct BuildAccelerationStructureNV : u32 {
          const ALLOW_UPDATE = 0x1;
          const ALLOW_COMPACTION = 0x2;
          const PREFER_FAST_TRACE = 0x4;
          const PREFER_FAST_BUILD = 0x8;
          const LOW_MEMORY = 0x10;
    }
}
bitflags! {
    struct PipelineCreationFeedbackEXT : u32 {
          const VALID = 0x1;
          const APPLICATION_PIPELINE_CACHE_HIT = 0x2;
          const BASE_PIPELINE_ACCELERATION = 0x4;
    }
}
bitflags! {
    struct ToolPurposeEXT : u32 {
          const VALIDATION = 0x1;
          const PROFILING = 0x2;
          const TRACING = 0x4;
          const ADDITIONAL_FEATURES = 0x8;
          const MODIFYING_FEATURES = 0x10;
    }
}
