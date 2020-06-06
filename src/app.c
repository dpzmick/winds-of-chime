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

static const uint32_t WIDTH  = 640;
static const uint32_t HEIGHT = 480;

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

  // check if we support triple buffering
  // if not, just pick FIFO, since it's always available
  VkPresentModeKHR picked = VK_PRESENT_MODE_FIFO_KHR;
  for( uint32_t i = 0; i < count; ++i ) {
    VkPresentModeKHR m = modes[i];
    if( m != VK_PRESENT_MODE_MAILBOX_KHR ) continue;
    LOG_INFO( "Enabling MAILBOX present mode" );
    picked = m;
    break;
  }

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
create_swapchain( VkPhysicalDevice phy,
                  VkDevice         device,
                  VkSurfaceKHR     surface,
                  uint32_t *       out_n_swapchain_images,
                  VkImage * *      out_swapchain_images,
                  VkImageView * *  out_image_views )
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

  uint32_t n_swapchain_images;

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

  *out_n_swapchain_images = n_swapchain_images;
  *out_swapchain_images   = swapchain_images;
  *out_image_views        = image_views;
  return swapchain;
}

static void
setup_images( app_t * app )
{
  // app is partially constructed
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

  uint32_t physical_device_count = 0;
  res = vkEnumeratePhysicalDevices( instance, &physical_device_count, NULL );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to enumerate devices ret=%d", res );
  }

  LOG_INFO( "Found %u physical devices", physical_device_count );

  size_t sz = physical_device_count * sizeof( VkPhysicalDevice );
  VkPhysicalDevice* physical_devices = malloc( sz );
  if( UNLIKELY( physical_devices == NULL ) ) {
    FATAL( "Failed to allocate physical devices" );
  }

  res = vkEnumeratePhysicalDevices( instance, &physical_device_count, physical_devices );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to enumerate physical devices the second time, ret=%d", res );
  }

  bool found_device = false;

  for( uint32_t i = 0; i < physical_device_count; ++i ) {
    VkPhysicalDevice dev = physical_devices[i];

    VkQueueFamilyProperties* props    = NULL;
    uint32_t                 prop_cnt = 0;

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, NULL );
    props = malloc( sizeof( *props ) * prop_cnt );
    if( !props ) FATAL( "Failed to allocate memory" );

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, props );

    // graphics implies transfer
    // need a graphics queue which supports present as well
    uint32_t graphics_queue = 0;
    bool     found_queue    = false;

    for( uint32_t j = 0; j < prop_cnt; ++j ) {
      VkQueueFlags flags = props[i].queueFlags;
      VkBool32     present;
      res = vkGetPhysicalDeviceSurfaceSupportKHR( dev, j, app->window_surface, &present );

      if( !present ) continue;
      if( !(flags & VK_QUEUE_GRAPHICS_BIT) ) continue;

      // forcing graphics queue and present queue to be same queue
      // FIXME this probably doens't work everywhere?
      graphics_queue = j;
      found_queue    = true;
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

    if( LIKELY( found_queue ) ) {
      // LOG_INFO( "Found memory at idx %u", memory_idx );
      LOG_INFO( "Graphics queue at idx %u", graphics_queue );

      open_device( app, dev, graphics_queue );
      // open_memory( &app->coherent_memory, app->device, memory_idx );
      // map_memory( &app->mapped_memory, app->device, app->coherent_memory );
      app->swapchain = create_swapchain( dev, app->device, app->window_surface,
                                         /* out */
                                         &app->n_swapchain_images,
                                         &app->swapchain_images,
                                         &app->image_views );

      // FIXME maybe store extent, tutorial says to..

      found_device = true;
      free( props );
      break;
    }
    else {
      bool found_mem = false;
      LOG_INFO( "Device %u not valid. found memory? %s found_queue %s", i,
                ( found_mem   ? "YES" : "NO" ),
                ( found_queue ? "YES" : "NO" ) );

    }

    free( props );
  }

  if( UNLIKELY( !found_device ) ) {
    FATAL( "No acceptable device found" );
  }

  free( physical_devices );

  // create swap chain (probably need to add an extension)
  // create image views

#if 0
  // continue init
  size_t shader_bytes = 0;
  char* shader_contents = read_entire_file( "src/shader.spv", &shader_bytes );
  if( UNLIKELY( shader_bytes % sizeof( uint32_t ) != 0 ) ) {
    FATAL( "Shader is the wrong size, should be uint32_t multiple" );
  }

  LOG_INFO( "Loaded shader, size=%zu", shader_bytes );

  VkShaderModuleCreateInfo screate[] = {{
      .sType    = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      .pNext    = NULL,
      .flags    = 0,
      .codeSize = shader_bytes,
      .pCode    = (uint32_t const*)shader_contents,
  }};

  res = vkCreateShaderModule( app->device, screate, NULL, &app->shader );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create shader module ret=%d", res );
  }

  free( shader_contents );

  VkBufferCreateInfo bcreate[] = {{
      .sType                 = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
      .pNext                 = NULL,
      .flags                 = 0,
      .size                  = BUFFER_SIZE,
      .usage                 = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT,
      .sharingMode           = VK_SHARING_MODE_EXCLUSIVE, // applies to vk queues only
      .queueFamilyIndexCount = 0,
      .pQueueFamilyIndices   = NULL,
  }};

  res = vkCreateBuffer( app->device, bcreate, 0, &app->in_buffer );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create in_buffer, ret=%d", res );
  }

  res = vkCreateBuffer( app->device, bcreate, 0, &app->out_buffer );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create out_buffer, ret=%d", res );
  }

  // bind buffers to first half and second half of memory
  res = vkBindBufferMemory( app->device, app->in_buffer, app->coherent_memory, 0 );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to bind in_bufer" );
  }

  res = vkBindBufferMemory( app->device, app->out_buffer, app->coherent_memory, BUFFER_SIZE );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to bind out_bufer" );
  }

  // use the buffers to create bindings for the shader
  VkDescriptorSetLayoutBinding bindings[] = {{
      .binding            = 0,
      .descriptorType     = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
      .descriptorCount    = 1,
      .stageFlags         = VK_SHADER_STAGE_COMPUTE_BIT,
      .pImmutableSamplers = NULL,
  }, {
      .binding            = 1,
      .descriptorType     = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
      .descriptorCount    = 1,
      .stageFlags         = VK_SHADER_STAGE_COMPUTE_BIT,
      .pImmutableSamplers = NULL,
  }};

  VkDescriptorSetLayoutCreateInfo slci[] = {{
      .sType        = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
      .pNext        = NULL,
      .flags        = 0,
      .bindingCount = ARRAY_SIZE( bindings ),
      .pBindings    = bindings,
  }};

  res = vkCreateDescriptorSetLayout( app->device, slci, NULL, &app->dset_layout );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create dset layout" );
  }

  VkPipelineLayoutCreateInfo plci[] = {{
      .sType                  = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
      .pNext                  = NULL,
      .flags                  = 0,
      .setLayoutCount         = 1,
      .pSetLayouts            = &app->dset_layout,
      .pushConstantRangeCount = 0,
      .pPushConstantRanges    = NULL,
  }};

  res = vkCreatePipelineLayout( app->device, plci, NULL, &app->playout );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create pipeline layout" );
  }

  VkPipelineShaderStageCreateInfo pssci[] = {{
      .sType               = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
      .pNext               = NULL,
      .flags               = 0,
      .stage               = VK_SHADER_STAGE_COMPUTE_BIT,
      .module              = app->shader,
      .pName               = "main",
      .pSpecializationInfo = NULL,
  }};

  VkComputePipelineCreateInfo pci[] = {{
      .sType              = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
      .pNext              = NULL,
      .flags              = 0,
      .stage              = *pssci,
      .layout             = app->playout,
      .basePipelineHandle = 0,
      .basePipelineIndex  = 0,
  }};

  res = vkCreateComputePipelines( app->device, VK_NULL_HANDLE, 1, pci, NULL, &app->pipeline );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create vulkan pipeline, ret=%d", res );
  }

  VkDescriptorPoolSize dpool_sizes[] = {{
      .type            = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
      .descriptorCount = 2,
  }};

  VkDescriptorPoolCreateInfo dpci[] = {{
      .sType         = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
      .pNext         = NULL,
      .flags         = 0,
      .maxSets       = 1,
      .poolSizeCount = 1,
      .pPoolSizes    = dpool_sizes,
  }};

  res = vkCreateDescriptorPool( app->device, dpci, NULL, &app->pool );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create descriptor pool" );
  }

  VkDescriptorSetAllocateInfo dsai[] = {{
      .sType              = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
      .pNext              = NULL,
      .descriptorPool     = app->pool,
      .descriptorSetCount = 1,
      .pSetLayouts        = &app->dset_layout,
  }};

  res = vkAllocateDescriptorSets( app->device, dsai, &app->dset );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to allocate dset" );
  }

  VkDescriptorBufferInfo in_info[] = {{
      .buffer = app->in_buffer,
      .offset = 0,
      .range = VK_WHOLE_SIZE,
  }};

  VkDescriptorBufferInfo out_info[] = {{
      .buffer = app->out_buffer,
      .offset = 0,
      .range = VK_WHOLE_SIZE,
  }};

  VkWriteDescriptorSet write_dset[] = {{
      .sType            = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      .pNext            = NULL,
      .dstSet           = app->dset,
      .dstBinding       = 0,
      .dstArrayElement  = 0,
      .descriptorCount  = 1,
      .descriptorType   = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
      .pImageInfo       = NULL,
      .pBufferInfo      = in_info,
      .pTexelBufferView = NULL,
  }, {
      .sType            = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      .pNext            = NULL,
      .dstSet           = app->dset,
      .dstBinding       = 1,
      .dstArrayElement  = 0,
      .descriptorCount  = 1,
      .descriptorType   = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
      .pImageInfo       = NULL,
      .pBufferInfo      = out_info,
      .pTexelBufferView = NULL,
  }};

  vkUpdateDescriptorSets( app->device, 2, write_dset, 0, NULL );

  // FIXME revist the descriptor bindings nonsense

  VkCommandPoolCreateInfo cmdpci[] = {{
      .sType            = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      .pNext            = NULL,
      .flags            = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
      .queueFamilyIndex = app->compute_queue_idx,
  }};

  res = vkCreateCommandPool( app->device, cmdpci, NULL, &app->cmd_pool );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create command pool" );
  }

  VkCommandBufferAllocateInfo cmdbci[] = {{
      .sType              = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
      .pNext              = NULL,
      .commandPool        = app->cmd_pool,
      .level              = VK_COMMAND_BUFFER_LEVEL_PRIMARY,
      .commandBufferCount = 1,
  }};

  res = vkAllocateCommandBuffers( app->device, cmdbci, &app->cmd_buffer );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to allocate command buffers" );
  }
#endif
}

void
app_destroy( app_t* app )
{
  if( !app ) return;
  // close the window first
  glfwDestroyWindow( app->window );

  // then tear down in dag order
  for( uint32_t i = 0; i < app->n_swapchain_images; ++i ) {
    vkDestroyImageView( app->device, app->image_views[i], NULL );
  }
  free( app->image_views );
  free( app->swapchain_images );
  vkDestroySwapchainKHR( app->device, app->swapchain, NULL );
  vkDestroySurfaceKHR( app->instance, app->window_surface, NULL );

  vkDestroyDevice( app->device, NULL );

#if 0
  /* cmd_buffer? */
  vkDestroyCommandPool( app->device, app->cmd_pool, NULL );
  /* dset? */ // FIXME this is a leak
  vkDestroyDescriptorPool( app->device, app->pool, NULL );
  vkDestroyPipeline( app->device, app->pipeline, NULL );
  vkDestroyPipelineLayout( app->device, app->playout, NULL );
  vkDestroyDescriptorSetLayout( app->device, app->dset_layout, NULL );
  vkDestroyBuffer( app->device, app->in_buffer, NULL );
  vkDestroyBuffer( app->device, app->out_buffer, NULL );
  vkDestroyShaderModule( app->device, app->shader, NULL );
  vkUnmapMemory( app->device, app->coherent_memory );
  vkFreeMemory( app->device, app->coherent_memory, NULL );
#endif
}

#if 0
__attribute__((noinline))
static uint64_t
run_once( app_t*  app,
          VkFence fence )
{
  // record a command
  VkCommandBufferBeginInfo cbbi[] = {{
      .sType            = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      .pNext            = NULL,
      .flags            = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      .pInheritanceInfo = NULL,
  }};

  /* Begin recording the command buffer */
  VkResult res = vkBeginCommandBuffer( app->cmd_buffer, cbbi );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to begin command buffer" );
  }

  vkCmdBindPipeline( app->cmd_buffer, VK_PIPELINE_BIND_POINT_COMPUTE, app->pipeline );
  vkCmdBindDescriptorSets( app->cmd_buffer, VK_PIPELINE_BIND_POINT_COMPUTE, app->playout, 0, 1, &app->dset, 0, NULL );
  vkCmdDispatch( app->cmd_buffer, 1, 1, 1 ); // run exactly one shader

  res = vkEndCommandBuffer( app->cmd_buffer );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to end command buffer" );
  }

  VkSubmitInfo submit_info[] = {{
      .sType              = VK_STRUCTURE_TYPE_SUBMIT_INFO,
      .pNext              = NULL,
      .commandBufferCount = 1,
      .pCommandBuffers    = &app->cmd_buffer,
  }};

  uint32_t volatile* mem = app->mapped_memory;
  uint32_t volatile* loc = mem + N_INTS;

  *mem = 0;
  *loc = 0;
  vkQueueSubmit( app->queue, 1, submit_info, fence ); // not sure when this returns?

  *mem = 1; // trigger the write
  uint64_t start = rdtscp();

  // wait for the two
  while( true ) {
    if( LIKELY( *loc == 2 ) ) break;
    if( LIKELY( *loc == 3 ) ) {
      LOG_INFO( "Trial failed" );
      return 0;
    }
  }

  uint64_t finish = rdtscp();

  res = vkWaitForFences( app->device, 1, &fence, VK_TRUE, 100000 );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to wait for fence" );
  }

  return finish-start;
}
#endif

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
  GLFWwindow* window = app->window;

  glfwSetMouseButtonCallback( window, mouse_button_callback );

  while( !glfwWindowShouldClose( window ) ) {
    glfwPollEvents();
  }

#if 0
  VkFenceCreateInfo fci[] = {{
      .sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
      .pNext = NULL,
      .flags = 0,
  }};

  VkFence fence;
  VkResult res = vkCreateFence( app->device, fci, NULL, &fence );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create fence, res=%d", res );
  }

  uint64_t trials[1024];
  for( size_t i = 0; i < ARRAY_SIZE( trials ); ++i ) {
    trials[i] = run_once( app, fence );
    res = vkResetFences( app->device, 1, &fence );
    if( UNLIKELY( res != VK_SUCCESS ) ) {
      FATAL( "Failed to reset fence" );
    }
  }

  vkDestroyFence( app->device, fence, NULL );
  
  static uint64_t tsc_freq_khz = 3892231; // AMD
  // static uint64_t tsc_freq_khz = 2099944; // intel
  double          ns_per_cycle = 1./((double)(tsc_freq_khz * 1000)/1e9);
  for( size_t i = 0; i < ARRAY_SIZE( trials ); ++i ) {
    printf( "%f\n", (double)trials[i]*ns_per_cycle );
  }
#endif
}
