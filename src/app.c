#include "app.h"
#include "common.h"
#include "log.h"

#include "volk.h"

#include <assert.h>
#include <GLFW/glfw3.h>
#include <errno.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static const uint32_t WIDTH  = 800;
static const uint32_t HEIGHT = 600;

static char*
read_entire_file( char const* filename,
                  size_t*     out_bytes )
{
  size_t mem_used  = 0;
  size_t mem_space = 4096;
  char*  buffer    = malloc( mem_space );
  if( UNLIKELY( !buffer ) ) FATAL( "Failed to allocate memory" );

  int fd = open( filename, O_RDONLY );
  if( UNLIKELY( fd < 0 ) ) FATAL( "Failed to open file %s", filename );

  while( 1 ) {
    size_t  to_read = mem_space-mem_used;
    ssize_t n_read  = read( fd, buffer+mem_used, to_read );
    if( n_read < 0 ) {
      FATAL( "Failed to read file errno=%d", errno );
    }

    if( n_read == 0 ) {
      close( fd );
      *out_bytes = mem_used;
      return buffer;
    }

    mem_used += (size_t)n_read;

    // we need a larger buffer
    mem_space = mem_space*2;
    buffer = realloc( buffer, mem_space );
    if( UNLIKELY( !buffer ) ) FATAL( "Failed to allocate memory" );
  }
}

static VkShaderModule
create_shader( char const * fname,
               VkDevice     device )
{
  VkResult       res;
  VkShaderModule shader;

  size_t shader_bytes    = 0;
  char*  shader_contents = read_entire_file( fname, &shader_bytes );
  if( UNLIKELY( shader_bytes % sizeof( uint32_t ) != 0 ) ) {
    FATAL( "Shader is the wrong size, should be uint32_t multiple" );
  }

  VkShaderModuleCreateInfo ci[] = {{
      .sType    = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      .pNext    = NULL,
      .flags    = 0,
      .codeSize = shader_bytes,
      .pCode    = (uint32_t const*)shader_contents,
  }};

  res = vkCreateShaderModule( device, ci, NULL, &shader );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create shader module ret=%d", res );
  }

  free( shader_contents );
  LOG_INFO( "Loaded shader from %s, size=%zu", fname, shader_bytes );

  return shader;
}

static VkPhysicalDevice*
get_physical_devices( VkInstance instance,
                      uint32_t * out_device_count )
{
  VkResult res;
  uint32_t          physical_device_count = 0;
  VkPhysicalDevice* physical_devices      = NULL;

  res = vkEnumeratePhysicalDevices( instance, &physical_device_count, NULL );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to enumerate devices ret=%d", res );
  }

  LOG_INFO( "Found %u physical devices", physical_device_count );

  physical_devices = malloc( physical_device_count * sizeof( *physical_devices ));
  if( UNLIKELY( physical_devices == NULL ) ) {
    FATAL( "Failed to allocate physical devices" );
  }

  res = vkEnumeratePhysicalDevices( instance, &physical_device_count, physical_devices );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to enumerate physical devices the second time, ret=%d", res );
  }

  *out_device_count = physical_device_count;
  return physical_devices;
}

static VkPhysicalDevice
select_physical_device( VkInstance   instance,
                        VkSurfaceKHR window_surface,
                        uint32_t *   out_queue_idx )
{
  VkResult                 res;
  VkQueueFamilyProperties* props    = NULL;
  uint32_t                 prop_cnt = 0;

  // outparams
  VkPhysicalDevice device         = VK_NULL_HANDLE;
  uint32_t         graphics_queue = (uint32_t)-1;

  uint32_t           device_count;
  VkPhysicalDevice * devices = get_physical_devices( instance, &device_count );

  for( uint32_t i = 0; i < device_count; ++i ) {
    VkPhysicalDevice dev = devices[i];

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, NULL );

    props = realloc( props, sizeof( *props ) * prop_cnt );
    if( !props ) FATAL( "Failed to allocate memory" );

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, props );

    // need queues for graphics, present, and transfer
    // for now, assuming that a queue with GRAPHICS_BIT implies all of the above
    bool found_queue = false;

    for( uint32_t j = 0; j < prop_cnt; ++j ) {
      VkQueueFlags flags = props[i].queueFlags;
      VkBool32     present;
      res = vkGetPhysicalDeviceSurfaceSupportKHR( dev, j, window_surface, &present );

      if( !present ) continue;
      if( !(flags & VK_QUEUE_GRAPHICS_BIT) ) continue;

      graphics_queue = j;
      found_queue = true;
      break;
    }

    /* uint32_t memory_idx = 0; */
    /* bool     found_mem  = false; */

    /* VkPhysicalDeviceMemoryProperties mem_props[1]; */
    /* vkGetPhysicalDeviceMemoryProperties( dev, mem_props ); */

    /* uint32_t            n_mem = mem_props->memoryTypeCount; */
    /* VkMemoryType const* mt    = mem_props->memoryTypes; */
    /* for( uint32_t j = 0; j < n_mem; ++j ) { */
    /*   if( mt[j].propertyFlags & VK_MEMORY_PROPERTY_HOST_COHERENT_BIT ) { */
    /*     memory_idx = j; */
    /*     found_mem = true; */
    /*     break; */
    /*   } */
    /* } */


    if( found_queue ) {
      device = dev;
      break;
    }
  }

  free( props );
  free( devices );

  if( UNLIKELY( device == VK_NULL_HANDLE ) ) {
    FATAL( "No acceptable device found" );
  }

  *out_queue_idx = graphics_queue;
  return device;
}

static void
open_device( app_t *          app,
             VkPhysicalDevice physical_device,
             uint32_t         queue_idx )
{
  app->queue_priority[0] = 1.0;
  app->queue_idx = queue_idx;

  const VkDeviceQueueCreateInfo q_create[] = {{
      .sType            = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
      .pNext            = NULL,
      .flags            = 0,
      .queueFamilyIndex = queue_idx,
      .queueCount       = 1,
      .pQueuePriorities = app->queue_priority,
  }};

  char const * const enabled_exts[] = {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME
  };

  const VkDeviceCreateInfo device_c[] = {{
      .sType                   = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
      .pNext                   = NULL,
      .flags                   = 0,
      .queueCreateInfoCount    = ARRAY_SIZE( q_create ),
      .pQueueCreateInfos       = q_create,
      .enabledLayerCount       = 0,
      .ppEnabledLayerNames     = NULL,
      .enabledExtensionCount   = (uint32_t)ARRAY_SIZE( enabled_exts ),
      .ppEnabledExtensionNames = enabled_exts,
      .pEnabledFeatures        = NULL,
  }};

  VkDevice device = VK_NULL_HANDLE;
  VkResult res = vkCreateDevice( physical_device, device_c, NULL, &device );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create device" );
  }

  volkLoadDevice( device );

  vkGetDeviceQueue( device, queue_idx, 0, &app->queue );
  app->device = device;
}

static void
open_memory( VkDeviceMemory* memory,
             VkDevice        device,
             uint32_t        memory_type_idx,
             uint32_t        sz )
{
  VkMemoryAllocateInfo info[] = {{
      .sType           = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
      .pNext           = NULL,
      .allocationSize  = sz,
      .memoryTypeIndex = memory_type_idx,
  }};

  VkResult res = vkAllocateMemory( device, info, NULL, memory );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to allocate memory on device, ret=%d", res );
  }
}

static void
map_memory( void volatile** map_to,
            uint32_t        sz,
            VkDevice        device,
            VkDeviceMemory  memory )
{
  VkResult res = vkMapMemory( device, memory, 0, sz, 0, (void**)map_to );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to map memory" );
  }
}

static VkSurfaceFormatKHR
pick_surface_format( VkPhysicalDevice device,
                     VkSurfaceKHR     surface )
{
  VkResult res;
  uint32_t format_count;
  res = vkGetPhysicalDeviceSurfaceFormatsKHR( device, surface, &format_count, NULL );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain surface formats, err=%d", res );
  }

  if( format_count == 0 ) {
    FATAL( "No swapchain surface formats available" );
  }

  // we prefer SRBG according to vulkan tutorial
  VkSurfaceFormatKHR * formats = malloc( format_count * sizeof( *formats ) );
  if( UNLIKELY( !formats ) ) FATAL( "Failed to allocate" );

  res = vkGetPhysicalDeviceSurfaceFormatsKHR( device, surface, &format_count, formats );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain surface formats, err=%d", res );
  }

  // default to first one if we don't find preferred
  VkSurfaceFormatKHR picked = formats[0];
  for( uint32_t i = 0; i < format_count; ++i ) {
    VkSurfaceFormatKHR f = formats[i];
    if( f.format != VK_FORMAT_B8G8R8A8_SRGB ) continue;
    if( f.colorSpace != VK_COLOR_SPACE_SRGB_NONLINEAR_KHR ) continue;
    picked = f;
    break;
  }

  free( formats );
  return picked;
}

static VkPresentModeKHR
pick_surface_present_mode( VkPhysicalDevice device,
                           VkSurfaceKHR     surface )
{
  VkResult res;
  uint32_t count;

  res = vkGetPhysicalDeviceSurfacePresentModesKHR( device, surface, &count, NULL );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain present modes, err=%d", res );
  }

  VkPresentModeKHR* modes = malloc( count * sizeof( *modes ) );
  if( UNLIKELY( !modes ) ) FATAL( "Failed to allocate" );

  res = vkGetPhysicalDeviceSurfacePresentModesKHR( device, surface, &count, modes );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain present modes, err=%d", res );
  }

  VkPresentModeKHR picked = VK_PRESENT_MODE_FIFO_KHR;

#if 0 // let's just run at 60 fps
  // check if we support triple buffering
  for( uint32_t i = 0; i < count; ++i ) {
    VkPresentModeKHR m = modes[i];
    if( m != VK_PRESENT_MODE_MAILBOX_KHR ) continue;
    LOG_INFO( "Enabling MAILBOX present mode" );
    picked = m;
    break;
  }
#endif

  free( modes );
  return picked;
}

// basically the size of the images that will be drawn by swapchain
static VkExtent2D
pick_swap_extent( VkSurfaceCapabilitiesKHR const* caps )
{
  if( caps->currentExtent.width == UINT32_MAX && caps->currentExtent.height == UINT32_MAX ) {
    return caps->currentExtent;
  }

  // otherwise, we pick the largest extent we can
  VkExtent2D actual = {
    .width  = WIDTH,
    .height = HEIGHT,
  };

  VkExtent2D min = caps->minImageExtent;
  VkExtent2D max = caps->maxImageExtent;

  actual.width  = MAX( min.width,  MIN( max.width,  actual.width ) );
  actual.height = MAX( min.height, MIN( max.height, actual.height ) );

  return actual;
}

static VkSwapchainKHR
create_swapchain( VkPhysicalDevice     phy,
                  VkDevice             device,
                  VkSurfaceKHR         surface,
                  uint32_t *           out_n_swapchain_images,
                  VkImage * *          out_swapchain_images,
                  VkSurfaceFormatKHR * out_surface_format,
                  VkImageView * *      out_image_views,
                  VkExtent2D *         out_extent )
{
  VkResult                 res;
  VkSurfaceFormatKHR       surface_format;
  VkPresentModeKHR         surface_present_mode;
  VkExtent2D               surface_swap_extent;
  uint32_t                 image_count;
  VkSurfaceCapabilitiesKHR caps[1];

  // make sure that we actually managed to load these functions
  assert( vkCreateSwapchainKHR );

  res = vkGetPhysicalDeviceSurfaceCapabilitiesKHR( phy, surface, caps );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain capabilites, err=%d", res );
  }

  surface_format       = pick_surface_format( phy, surface );
  surface_present_mode = pick_surface_present_mode( phy, surface );
  surface_swap_extent  = pick_swap_extent( caps );
  image_count          = caps->minImageCount + 1;

  // if zero, there's no max
  if( caps->maxImageCount ) image_count = MIN( image_count, caps->maxImageCount );

  VkSwapchainCreateInfoKHR ci[1] = {{
    .sType                 = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
    .pNext                 = NULL,
    .flags                 = 0,
    .surface               = surface,
    .minImageCount         = image_count,
    .imageFormat           = surface_format.format,
    .imageColorSpace       = surface_format.colorSpace,
    .imageExtent           = surface_swap_extent,
    .imageArrayLayers      = 1,
    .imageUsage            = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT, // render directly to these for now
    .imageSharingMode      = VK_SHARING_MODE_EXCLUSIVE, // graphics queue and present queue are same queue
    .queueFamilyIndexCount = 0, // ignored
    .pQueueFamilyIndices   = NULL,
    .preTransform          = caps->currentTransform,
    .compositeAlpha        = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
    .presentMode           = surface_present_mode,
    .clipped               = VK_TRUE,
    .oldSwapchain          = VK_NULL_HANDLE, // if recreating, pass this
  }};

  VkSwapchainKHR swapchain;
  res = vkCreateSwapchainKHR( device, ci, NULL, &swapchain );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create swapchain, err=%d", res );
  }

  uint32_t n_swapchain_images; // could be greater than image_count

  res = vkGetSwapchainImagesKHR( device, swapchain, &n_swapchain_images, NULL );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain images, err=%d", res );
  }

  VkImage* swapchain_images = malloc( n_swapchain_images * sizeof( *swapchain_images ) );
  if( UNLIKELY( !swapchain_images ) ) FATAL( "Failed to allocate" );

  res = vkGetSwapchainImagesKHR( device, swapchain, &n_swapchain_images, swapchain_images );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to get swapchain images, err=%d", res );
  }

  VkImageView* image_views = malloc( n_swapchain_images * sizeof( *image_views ) );
  if( UNLIKELY( !image_views ) ) FATAL( "Failed to allocate" );

  for( uint32_t i = 0; i < n_swapchain_images; ++i ) {
    VkImageViewCreateInfo ci[1] = {{
      .sType        = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
      .pNext        = NULL,
      .image        = swapchain_images[i],
      .viewType     = VK_IMAGE_VIEW_TYPE_2D,
      .format       = surface_format.format,
      /* don't screw with the colors */
      .components.r = VK_COMPONENT_SWIZZLE_IDENTITY,
      .components.g = VK_COMPONENT_SWIZZLE_IDENTITY,
      .components.b = VK_COMPONENT_SWIZZLE_IDENTITY,
      .components.a = VK_COMPONENT_SWIZZLE_IDENTITY,
      /* ???? */
      .subresourceRange.aspectMask     = VK_IMAGE_ASPECT_COLOR_BIT,
      .subresourceRange.baseMipLevel   = 0,
      .subresourceRange.levelCount     = 1,
      .subresourceRange.baseArrayLayer = 0,
      .subresourceRange.layerCount     = 1,
    }};

    res = vkCreateImageView( device, ci, NULL, &image_views[i] );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to create image view, err=%d", res );
    }
  }

  LOG_INFO( "Using %u swapchain images", n_swapchain_images );

  *out_extent             = surface_swap_extent;
  *out_n_swapchain_images = n_swapchain_images;
  *out_swapchain_images   = swapchain_images;
  *out_surface_format     = surface_format;
  *out_image_views        = image_views;
  return swapchain;
}

static GLFWwindow*
open_window( void )
{
  // don't create any API context, not needed for vulkan
  glfwWindowHint( GLFW_CLIENT_API, GLFW_NO_API );

  glfwWindowHint( GLFW_RESIZABLE, GLFW_FALSE );
  glfwWindowHint( GLFW_FLOATING,  GLFW_TRUE );

  // not sure how this interfacts with vulkan FIXME figure out
  glfwWindowHint( GLFW_SCALE_TO_MONITOR, GLFW_TRUE );

  GLFWwindow* window = glfwCreateWindow( (int)WIDTH, (int)HEIGHT, "Winds of Chime", NULL, NULL );
  if( !window ) {
    char const * desc;
    int err = glfwGetError( &desc );
    FATAL( "Failed to open GLFW window %s (%d)", desc, err );
  }

  return window;
}

void
app_init( app_t*     app,
          VkInstance instance )
{
  VkResult res;

  app->instance = instance;
  app->window = open_window();
  res = glfwCreateWindowSurface( app->instance, app->window, NULL, &app->window_surface );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create vulkan surface, err=%d", res );
  }

  glfwSetWindowUserPointer( app->window, app );

  uint32_t queue_idx;
  VkPhysicalDevice physical_device = select_physical_device( instance,
                                                             app->window_surface,
                                                             &queue_idx );

  LOG_INFO( "Graphics queue at idx %u", queue_idx );

  open_device( app, physical_device, queue_idx );
  // open_memory( &app->coherent_memory, app->device, memory_idx );
  // map_memory( &app->mapped_memory, app->device, app->coherent_memory );
  app->swapchain = create_swapchain( physical_device, app->device, app->window_surface,
                                     /* out */
                                     &app->n_swapchain_images,
                                     &app->swapchain_images,
                                     app->swapchain_surface_format,
                                     &app->image_views,
                                     app->swapchain_extent );

  // it's pipelining time

  app->vert = create_shader( "/home/dpzmick/programming/winds-of-chime/build/src/vert.spv", app->device );
  app->frag = create_shader( "/home/dpzmick/programming/winds-of-chime/build/src/frag.spv", app->device );

  VkPipelineShaderStageCreateInfo shader_stages[] = {{
    .sType               = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
    .pNext               = NULL,
    .flags               = 0,
    .stage               = VK_SHADER_STAGE_VERTEX_BIT,
    .module              = app->vert,
    .pName               = "main",
    .pSpecializationInfo = NULL,
  }, {
    .sType               = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
    .pNext               = NULL,
    .flags               = 0,
    .stage               = VK_SHADER_STAGE_FRAGMENT_BIT,
    .module              = app->frag,
    .pName               = "main",
    .pSpecializationInfo = NULL,
  }};

  // describe the vertex data that is input to the vertex shader
  // for now, basically empty
  VkPipelineVertexInputStateCreateInfo vert_input_ci[] = {{
    .sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
    .pNext = NULL,
    .flags = 0,
    .vertexBindingDescriptionCount   = 0,
    .pVertexBindingDescriptions      = NULL,
    .vertexAttributeDescriptionCount = 0,
    .pVertexAttributeDescriptions    = NULL,
  }};

  // describe what kind of geometry will be drawn
  VkPipelineInputAssemblyStateCreateInfo input_astate_ci[] = {{
    .sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
    .pNext = 0,
    .flags = 0,
    .topology               = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
    .primitiveRestartEnable = VK_FALSE,
  }};

  VkViewport viewport[1] = {{
    .x        = 0.0f,
    .y        = 0.0f,
    .width    = (float)app->swapchain_extent->width,
    .height   = (float)app->swapchain_extent->height,
    .minDepth = 0.0f,
    .maxDepth = 1.0f,
  }};

  VkRect2D scissor[] = {{
    .offset = (VkOffset2D){.x = 0, .y = 0},
    .extent = *app->swapchain_extent,
  }};

  VkPipelineViewportStateCreateInfo viewport_ci[] = {{
    .sType         = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
    .pNext         = NULL,
    .flags         = 0,
    .viewportCount = 1,
    .pViewports    = viewport,
    .scissorCount  = 1,
    .pScissors     = scissor,
  }};

  VkPipelineRasterizationStateCreateInfo raster_ci[] = {{
    .sType                   = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
    .pNext                   = NULL,
    .flags                   = 0,
    .depthClampEnable        = VK_FALSE,
    .rasterizerDiscardEnable = VK_FALSE,
    .polygonMode             = VK_POLYGON_MODE_FILL,
    .cullMode                = VK_CULL_MODE_BACK_BIT,
    .frontFace               = VK_FRONT_FACE_CLOCKWISE,
    .depthBiasEnable         = VK_FALSE,
    .depthBiasConstantFactor = 0.0f,
    .depthBiasClamp          = 0.0f,
    .depthBiasSlopeFactor    = 0.0f,
    .lineWidth               = 1.0f,
  }};

  VkPipelineMultisampleStateCreateInfo msaa[] = {{
    .sType                 = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
    .pNext                 = NULL,
    .flags                 = 0,
    .rasterizationSamples  = VK_SAMPLE_COUNT_1_BIT,
    .sampleShadingEnable   = VK_FALSE,
    .minSampleShading      = 1.0f,
    .pSampleMask           = NULL,
    .alphaToCoverageEnable = VK_FALSE,
    .alphaToOneEnable      = VK_FALSE,
  }};

  VkPipelineColorBlendAttachmentState color_blend_attach[1];
  memset( color_blend_attach, 0, sizeof( color_blend_attach ) );
  color_blend_attach->colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
  color_blend_attach->blendEnable    = VK_FALSE;

  VkPipelineColorBlendStateCreateInfo blend_ci[] = {{
    .sType           = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
    .pNext           = NULL,
    .flags           = 0,
    .logicOpEnable   = VK_FALSE,
    .logicOp         = VK_LOGIC_OP_COPY,
    .attachmentCount = 1,
    .pAttachments    = color_blend_attach,
    .blendConstants  = {0.0f, 0.0f, 0.0f, 0.0f},
  }};

  VkPipelineLayoutCreateInfo pipeline_layout_ci[] = {{
    .sType                  = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
    .pNext                  = NULL,
    .flags                  = 0,
    .setLayoutCount         = 0,
    .pSetLayouts            = NULL,
    .pushConstantRangeCount = 0,
    .pPushConstantRanges    = NULL,
  }};

  res = vkCreatePipelineLayout( app->device, pipeline_layout_ci, NULL, &app->pipeline_layout );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create pipeline layout, err=%d", res );
  }

  // -- render pass

  VkAttachmentDescription color_attach[] = {{
    .flags          = 0,
    .format         = app->swapchain_surface_format->format,
    .samples        = VK_SAMPLE_COUNT_1_BIT,
    .loadOp         = VK_ATTACHMENT_LOAD_OP_CLEAR,
    .storeOp        = VK_ATTACHMENT_STORE_OP_STORE,
    .stencilLoadOp  = VK_ATTACHMENT_LOAD_OP_DONT_CARE,
    .stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE,
    .initialLayout  = VK_IMAGE_LAYOUT_UNDEFINED,
    .finalLayout    = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
  }};

  VkAttachmentReference attach_ref[] = {{
    .attachment = 0,
    .layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
  }};

  VkSubpassDependency dependency[] = {{
    .srcSubpass      = VK_SUBPASS_EXTERNAL,
    .dstSubpass      = 0,
    .srcStageMask    = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
    .srcAccessMask   = 0,
    .dstStageMask    = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
    .dstAccessMask   = 0,
    .dependencyFlags = 0,
  }};

  VkSubpassDescription subpass[] = {{
    .flags                   = 0,
    .pipelineBindPoint       = VK_PIPELINE_BIND_POINT_GRAPHICS,
    .inputAttachmentCount    = 0,
    .pInputAttachments       = NULL,
    .colorAttachmentCount    = 1,
    .pColorAttachments       = attach_ref,
    .pResolveAttachments     = NULL,
    .pDepthStencilAttachment = NULL,
    .preserveAttachmentCount = 0,
    .pPreserveAttachments    = NULL,
  }};

  VkRenderPassCreateInfo render_pass_ci[] = {{
    .sType           = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
    .pNext           = NULL,
    .flags           = 0,
    .attachmentCount = 1,
    .pAttachments    = color_attach,
    .subpassCount    = 1,
    .pSubpasses      = subpass,
    .dependencyCount = 1,
    .pDependencies   = dependency,
  }};

  res = vkCreateRenderPass( app->device, render_pass_ci, NULL, &app->render_pass );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create render pass, err=%d", res );
  }

  // -- pipeline
  VkGraphicsPipelineCreateInfo pipeline_ci[] = {{
    .sType               = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
    .pNext               = NULL,
    .flags               = 0,
    .stageCount          = 2,
    .pStages             = shader_stages,
    .pVertexInputState   = vert_input_ci,
    .pInputAssemblyState = input_astate_ci,
    .pTessellationState  = NULL,
    .pViewportState      = viewport_ci,
    .pRasterizationState = raster_ci,
    .pMultisampleState   = msaa,
    .pDepthStencilState  = NULL,
    .pColorBlendState    = blend_ci,
    .pDynamicState       = NULL,
    .layout              = app->pipeline_layout,
    .renderPass          = app->render_pass,
    .subpass             = 0,
    .basePipelineHandle  = VK_NULL_HANDLE,
    .basePipelineIndex   = -1,
  }};

  res = vkCreateGraphicsPipelines( app->device, VK_NULL_HANDLE, 1, pipeline_ci, NULL, &app->graphics_pipeline );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create graphics pipeline, err=%d", res );
  }

  app->framebuffers = malloc( app->n_swapchain_images * sizeof( *app->framebuffers) );
  if( UNLIKELY( !app->framebuffers ) ) {
    FATAL( "Failed to allocate" );
  }

  for( uint32_t i = 0; i < app->n_swapchain_images; ++i ) {
    VkImageView attachments[] = { app->image_views[i] };
    VkFramebufferCreateInfo framebuffer_ci[] = {{
      .sType           = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
      .pNext           = NULL,
      .flags           = 0,
      .renderPass      = app->render_pass,
      .attachmentCount = 1,
      .pAttachments    = attachments,
      .width           = app->swapchain_extent->width,
      .height          = app->swapchain_extent->height,
      .layers          = 1,
    }};

    VkFramebuffer* out = &app->framebuffers[i];
    res = vkCreateFramebuffer( app->device, framebuffer_ci, NULL, out );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to create framebuffer, err=%d", res );
    }
  }

  // --
  VkCommandPoolCreateInfo command_pool_ci[] = {{
    .sType            = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
    .pNext            = NULL,
    .flags            = 0,
    .queueFamilyIndex = app->queue_idx,
  }};

  res = vkCreateCommandPool( app->device, command_pool_ci, NULL, &app->command_pool );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create command pool, err=%d", res );
  }

  app->command_buffers = malloc( app->n_swapchain_images * sizeof( *app->command_buffers ) );
  if( UNLIKELY( !app->command_buffers ) ) {
    FATAL( "Failed to allocate" );
  }

  VkCommandBufferAllocateInfo cb_ci[] = {{
    .sType              = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
    .pNext              = NULL,
    .commandPool        = app->command_pool,
    .level              = VK_COMMAND_BUFFER_LEVEL_PRIMARY,
    .commandBufferCount = app->n_swapchain_images, // one draw command for each swapchain image
  }};

  res = vkAllocateCommandBuffers( app->device, cb_ci, app->command_buffers );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to allocate command buffers, err=%d", res );
  }

  for( uint32_t i = 0; i < app->n_swapchain_images; ++i ) {
    // prerecord the commands for drawing
    VkCommandBuffer buffer = app->command_buffers[i];
    VkCommandBufferBeginInfo begin_info[] = {{
      .sType            = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      .pNext            = NULL,
      .flags            = 0,
      .pInheritanceInfo = NULL,
    }};

    res = vkBeginCommandBuffer( buffer, begin_info );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to begin command buffer, err=%d", res );
    }

    // nasty
    VkClearValue clear[] = {{
      .color = {
        .float32 = { 0.0f, 0.0f, 0.0f, 1.0f }
      }
    }};

    VkRenderPassBeginInfo render_pass_info[] = {{
      .sType           = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
      .pNext           = NULL,
      .renderPass      = app->render_pass,
      .framebuffer     = app->framebuffers[i],
      .renderArea      = { .offset = {0, 0}, .extent = *app->swapchain_extent },
      .clearValueCount = 1,
      .pClearValues    = clear,
    }};

    vkCmdBeginRenderPass( buffer, render_pass_info, VK_SUBPASS_CONTENTS_INLINE );
    vkCmdBindPipeline( buffer, VK_PIPELINE_BIND_POINT_GRAPHICS, app->graphics_pipeline );
    vkCmdDraw( buffer, 3, 1, 0, 0 );
    vkCmdEndRenderPass( buffer );

    res = vkEndCommandBuffer( buffer );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to end command buffer, err=%d", res );
    }
  }

  app->max_frames_in_flight = app->n_swapchain_images;
  LOG_INFO( "Using %u in flight frames", app->max_frames_in_flight );

  app->image_avail_semaphores = malloc( app->max_frames_in_flight * sizeof( VkSemaphore ) );
  if( UNLIKELY( !app->image_avail_semaphores ) ) {
    FATAL( "Failed to allocate" );
  }

  app->render_finished_semaphores = malloc( app->max_frames_in_flight * sizeof( VkSemaphore ) );
  if( UNLIKELY( !app->render_finished_semaphores ) ) {
    FATAL( "Failed to allocate" );
  }

  app->in_flight_fences = malloc( app->max_frames_in_flight * sizeof( VkSemaphore ) );
  if( UNLIKELY( !app->in_flight_fences) ) {
    FATAL( "Failed to allocate" );
  }

  app->images_in_flight = malloc( app->n_swapchain_images * sizeof( VkFence ) );
  if( UNLIKELY( !app->images_in_flight ) ) {
    FATAL( "Failed to allocate" );
  }

  for( uint32_t i = 0; i < app->max_frames_in_flight; ++i ) {
    VkSemaphoreCreateInfo sci[] = {{
      .sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
      .pNext = NULL,
      .flags = 0,
    }};

    VkFenceCreateInfo fci[] = {{
      .sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
      .pNext = NULL,
      .flags = VK_FENCE_CREATE_SIGNALED_BIT, // start off "ready"
    }};

    res = vkCreateSemaphore( app->device, sci, NULL, app->image_avail_semaphores + i );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to create semaphore, err=%d", res );
    }

    res = vkCreateSemaphore( app->device, sci, NULL, app->render_finished_semaphores + i);
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to create semaphore, err=%d", res );
    }

    res = vkCreateFence( app->device, fci, NULL, app->in_flight_fences + i );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to create fence, err=%d", res );
    }
  }

  // tracks if an image is currently being used by a previous frame render
  // NULL_HANDLE to indicate that an image is not in use
  for( uint32_t i = 0; i < app->n_swapchain_images; ++i ) {
    app->images_in_flight[i] = VK_NULL_HANDLE;
  }

  // we did it
}

void
app_destroy( app_t* app )
{
  if( !app ) return;

  for( uint32_t i = 0; i < app->max_frames_in_flight; ++i ) {
    vkDestroySemaphore( app->device, app->image_avail_semaphores[i], NULL );
    vkDestroySemaphore( app->device, app->render_finished_semaphores[i], NULL );
    vkDestroyFence( app->device, app->in_flight_fences[i], NULL );
  }

  free( app->images_in_flight );
  free( app->in_flight_fences );
  free( app->render_finished_semaphores );

  // images_in_flight is only storing references to frame semaphores, no cleanup
  // needed

  // then tear down in dag order
  for( uint32_t i = 0; i < app->n_swapchain_images; ++i ) {
    vkDestroyImageView( app->device, app->image_views[i], NULL );
  }

  free( app->image_views );
  free( app->swapchain_images );
  vkDestroySwapchainKHR( app->device, app->swapchain, NULL );
  vkDestroySurfaceKHR( app->instance, app->window_surface, NULL );

  vkDestroyShaderModule( app->device, app->vert, NULL );
  vkDestroyShaderModule( app->device, app->frag, NULL );

  vkDestroyRenderPass( app->device, app->render_pass, NULL );
  vkDestroyPipelineLayout( app->device, app->pipeline_layout, NULL );
  vkDestroyPipeline( app->device, app->graphics_pipeline, NULL );

  for( uint32_t i = 0; i < app->n_swapchain_images; ++i ) {
    vkDestroyFramebuffer( app->device, app->framebuffers[i], NULL );
  }
  free( app->framebuffers );

  vkDestroyCommandPool( app->device, app->command_pool, NULL );

  free( app->command_buffers );

  vkDestroyDevice( app->device, NULL );

  // close the window first
  glfwDestroyWindow( app->window );
}

static void
mouse_button_callback( GLFWwindow* window,
                       int         button,
                       int         action,
                       int         mods )
{
  if( button != GLFW_MOUSE_BUTTON_LEFT ) return;
  if( action != GLFW_PRESS )             return;

  // app_t* app = glfwGetWindowUserPointer( app );

  double x, y;
  glfwGetCursorPos( window, &x, &y );

  LOG_INFO( "button pressed at (%f,%f)", x, y );
}

void
app_run( app_t* app )
{
  VkResult res;

  // save invariants for better codegen
  VkDevice               device                     = app->device;
  VkQueue                queue                      = app->queue;
  VkSwapchainKHR         swapchain                  = app->swapchain;
  VkCommandBuffer* const commands                   = app->command_buffers;
  GLFWwindow*      const window                     = app->window;
  const uint32_t         max_frames_in_flight       = app->max_frames_in_flight;
  VkSemaphore* const     image_avail_semaphores     = app->image_avail_semaphores;
  VkSemaphore* const     render_finished_semaphores = app->render_finished_semaphores;
  VkFence* const         in_flight_fences           = app->in_flight_fences;
  VkFence* const         images_in_flight           = app->images_in_flight;

  glfwSetMouseButtonCallback( window, mouse_button_callback );

  uint64_t current_frame = 0;
  while( !glfwWindowShouldClose( window ) ) {
    glfwPollEvents();

    uint32_t             image_index     = 0;
    VkSemaphore          image_avail     = image_avail_semaphores[current_frame];
    VkSemaphore          render_finished = render_finished_semaphores[current_frame];
    VkFence              fence           = in_flight_fences[current_frame];
    VkPipelineStageFlags wait_stages[]   = { VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT };
    VkSemaphore          wait_sems[]     = { image_avail };
    VkSemaphore          signal_sems[]   = { render_finished };

    // acquireNextImage returns the index of the next image that we should use.
    // The image isn't actually available until the semaphore becomes ready.
    // this won't return 1,2,3,4,5,1,2,3,4,5,1,2,3,4,5.. seems to go in arbitrary order
    // the image returned here might already be in use by another frame
    // if it is, we'll also need to wait for the other frame to finish

    res = vkAcquireNextImageKHR( device, swapchain, UINT64_MAX, image_avail, VK_NULL_HANDLE, &image_index );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to acquire image, err=%d", res );
    }

    // make sure the last render of this frame is finished
    res = vkWaitForFences( device, 1, &fence, VK_TRUE, UINT64_MAX );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to wait for fence, err=%d", res );
    }

    // these values all depend on the image which was selected
    VkFence image_in_use = images_in_flight[image_index];

    VkSubmitInfo submit[] = {{
      .sType                = VK_STRUCTURE_TYPE_SUBMIT_INFO,
      .pNext                = NULL,
      .waitSemaphoreCount   = ARRAY_SIZE( wait_sems ),
      .pWaitSemaphores      = wait_sems,
      .pWaitDstStageMask    = wait_stages,
      .commandBufferCount   = 1,
      .pCommandBuffers      = commands + image_index,
      .signalSemaphoreCount = ARRAY_SIZE( signal_sems ),
      .pSignalSemaphores    = signal_sems,
    }};

    // depends on address of image index (technically could be setup sooner)
    VkPresentInfoKHR present_info[] = {{
      .sType              = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
      .pNext              = NULL,
      .waitSemaphoreCount = ARRAY_SIZE( signal_sems ),
      .pWaitSemaphores    = signal_sems,
      .swapchainCount     = 1,
      .pSwapchains        = &swapchain,
      .pImageIndices      = &image_index,
      .pResults           = NULL,
    }};

    // make sure the image is not in use by any other frames
    if( image_in_use != VK_NULL_HANDLE ) {
      // NOTE: this is a reference to some frame's fence
      // NOTE: if image_idx == frame_idx, we'll have waited on the frame fence
      // twice. this is okay, since we havne't reset the fence yet.
      res = vkWaitForFences( device, 1, &image_in_use, VK_TRUE, UINT64_MAX );
      if( UNLIKELY( res != VK_SUCCESS ) ) {
        FATAL( "Failed to wait for fence, err=%d", res );
      }
    }

    // clear frame fence
    res = vkResetFences( device, 1, &fence );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to reset fence, err=%d", res );
    }

    res = vkQueueSubmit( queue, 1, submit, fence );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to submit to queue, err=%d", res );
    }

    res = vkQueuePresentKHR( queue, present_info );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to present image" );
    }

    // save the fence for future frames that select this image
    images_in_flight[image_index] = fence;

    current_frame += 1;
    if( current_frame >= max_frames_in_flight ) current_frame = 0; /* cmov */
  }

  // wait for all outstanding requests to finish
  vkDeviceWaitIdle( device );
}
