#include "app.h"
#include "common.h"
#include "log.h"

#include "volk.h"

#include <GLFW/glfw3.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define N_INTS      128ul
#define BUFFER_SIZE ( N_INTS * sizeof( uint32_t ) )
#define MEMORY_SIZE ( 2ul * BUFFER_SIZE )

static char*
read_entire_file( char const* filename,
                  size_t*     out_bytes )
{
  size_t mem_used  = 0;
  size_t mem_space = 4096;
  char*  buffer    = malloc( mem_space );
  if( UNLIKELY( !buffer ) ) FATAL( "Failed to allocate memory" );

  // FIXME no reason for FILE* here, just use open
  FILE* f = fopen( filename, "r" );
  if( UNLIKELY( !f ) ) FATAL( "Failed to open file %s", filename );

  while( 1 ) {
    size_t to_read = mem_space-mem_used;
    size_t n_read = fread( buffer+mem_used, 1, to_read, f );
    mem_used += n_read;

    if( n_read < to_read ) {
      if( feof( f ) ) {
        fclose( f );
        *out_bytes = mem_used;
        return buffer;
      }
      else {
        FATAL( "Failed to read file errno=%d", ferror( f ) );
      }
    }

    // we need a larger buffer
    mem_space = mem_space*2;
    buffer = realloc( buffer, mem_space );
    if( UNLIKELY( !buffer ) ) FATAL( "Failed to allocate memory" );
  }
}

static void
open_device( app_t*           app,
             VkPhysicalDevice physical_device,
             uint32_t         queue_idx )
{
  VkResult res    = VK_SUCCESS;
  VkDevice device = VK_NULL_HANDLE;

  app->queue_priority = 1.0;
  app->queue_idx      = queue_idx;

  const VkDeviceQueueCreateInfo q_create[] = {{
      .sType            = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
      .pNext            = NULL,
      .flags            = 0,
      .queueFamilyIndex = queue_idx,
      .queueCount       = 1,
      .pQueuePriorities = &app->queue_priority,
  }};

  const VkDeviceCreateInfo device_c[] = {{
      .sType                   = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
      .pNext                   = NULL,
      .flags                   = 0,
      .queueCreateInfoCount    = ARRAY_SIZE( q_create ),
      .pQueueCreateInfos       = q_create,
      .enabledLayerCount       = 0,
      .ppEnabledLayerNames     = NULL,
      .enabledExtensionCount   = 0,
      .ppEnabledExtensionNames = NULL,
      .pEnabledFeatures        = NULL,
  }};

  res = vkCreateDevice( physical_device, device_c, NULL, &app->device );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create device" );
  }

  volkLoadDevice( app->device );
  vkGetDeviceQueue( app->device, queue_idx, 0, &app->queue );
}

static void
create_pools( app_t* app )
{
  VkResult res = VK_SUCCESS;

  VkCommandPoolCreateInfo cmdpci[] = {{
      .sType            = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      .pNext            = NULL,
      .flags            = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
      .queueFamilyIndex = app->queue_idx,
  }};

  res = vkCreateCommandPool( app->device, cmdpci, NULL, &app->cmd_pool );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create command pool" );
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

  res = vkCreateDescriptorPool( app->device, dpci, NULL, &app->descriptor_pool );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create descriptor pool" );
  }
}

static VkDeviceMemory
allocate_memory( VkDevice     device,
                 uint32_t     memory_type_idx,
                 VkDeviceSize size )
{
  VkResult       res = VK_SUCCESS;
  VkDeviceMemory ret = VK_NULL_HANDLE;

  VkMemoryAllocateInfo info[] = {{
      .sType           = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
      .pNext           = NULL,
      .allocationSize  = size,
      .memoryTypeIndex = memory_type_idx,
  }};

  res = vkAllocateMemory( device, info, NULL, &ret );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to allocate memory on device, ret=%d", res );
  }

  return ret;
}

static void*
map_memory( VkDevice        device,
            VkDeviceMemory  memory )
{
  void* ret;
  VkResult res = vkMapMemory( device, memory, 0, VK_WHOLE_SIZE, 0, &ret );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to map memory" );
  }

  return ret;
}

void
app_init( app_t*      app,
          VkInstance  instance,
          GLFWwindow* window)
{
  VkResult res;

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

  /* things we are looking for */
  uint32_t phys_idx       = 0;
  bool     found_queue    = false;
  uint32_t graphics_queue = 0;
  bool     found_host_mem = false;
  uint32_t host_idx       = 0;
  bool     found_card_mem = false;
  uint32_t card_idx       = 0;      // NOTE: might be the same as host, nbd if so

  for( ; phys_idx < physical_device_count; ++phys_idx ) {
    VkPhysicalDevice dev = physical_devices[phys_idx];

    VkQueueFamilyProperties* props    = NULL;
    uint32_t                 prop_cnt = 0;

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, NULL );
    props = malloc( sizeof( *props ) * prop_cnt );
    if( !props ) FATAL( "Failed to allocate memory" );

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, props );

    // NOTE: graphics implies transfer, drivers not required to mark
    //       as transfer
    // NOTE: currently requiring that graphics+present queue are same
    //       queue

    for( uint32_t j = 0; j < prop_cnt; ++j ) {
      VkQueueFlags flags = props[j].queueFlags;
      if( !(flags & VK_QUEUE_GRAPHICS_BIT) ) continue;
      if( !glfwGetPhysicalDevicePresentationSupport( instance, dev, j ) ) continue;

      graphics_queue = j;
      found_queue    = true;
      break;
    }

    VkPhysicalDeviceMemoryProperties mem_props[1];
    vkGetPhysicalDeviceMemoryProperties( dev, mem_props );

    uint32_t            n_mem = mem_props->memoryTypeCount;
    VkMemoryType const* mt    = mem_props->memoryTypes;
    for( uint32_t j = 0; j < n_mem; ++j ) {
      if( found_card_mem && found_host_mem ) break;
      if( mt[j].propertyFlags & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT ) {
        host_idx       = j;
        found_host_mem = true;
      }

      if( mt[j].propertyFlags & VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT ) {
        card_idx       = j;
        found_card_mem = true;
      }
    }

    if( LIKELY( found_queue && found_host_mem && found_card_mem ) ) {
      goto finish_init;
    }
    else {
      LOG_INFO( "Device %u not valid. found host memory %s found card memory %s found_queue %s", phys_idx,
                ( found_host_mem ? "YES" : "NO" ),
                ( found_card_mem ? "YES" : "NO" ),
                ( found_queue    ? "YES" : "NO" ) );

    }

    free( props );
  }

  /* if we get here, we didn't find a good device */
  FATAL( "No acceptable device found" );

finish_init:;
  VkPhysicalDevice phy = physical_devices[phys_idx];
  free( physical_devices ); // done with this

  LOG_INFO( "Compute queue at idx %u", graphics_queue );
  LOG_INFO( "Found host memory at idx %u", host_idx );
  LOG_INFO( "Found card memory at idx %u", card_idx );

  open_device( app, phy, graphics_queue );
  create_pools( app );

  app->host_memory = allocate_memory( app->device, host_idx, 2<<22 );
  app->mapped_host_memory = map_memory( app->device, app->host_memory );
  app->device_memory = allocate_memory( app->device, card_idx, 2<<22 );

  // we'll need to create some buffers on the gpu side so that we can
  // bind stuff to the gpu. whats the difference between an image and
  // a buffer?

  // also a swap chain ugh

  // also load shaders

  // build bindings

  // build pipeline

  // recording commang buffer happens where?

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

#if 0
  /* cmd_buffer? */
  /* dset? */
  vkDestroyPipeline( app->device, app->pipeline, NULL );
  vkDestroyPipelineLayout( app->device, app->playout, NULL );
  vkDestroyDescriptorSetLayout( app->device, app->dset_layout, NULL );
  vkDestroyBuffer( app->device, app->in_buffer, NULL );
  vkDestroyBuffer( app->device, app->out_buffer, NULL );
  vkDestroyShaderModule( app->device, app->shader, NULL );
  vkUnmapMemory( app->device, app->coherent_memory );
#endif

  vkDestroyCommandPool( app->device, app->cmd_pool, NULL );
  vkDestroyDescriptorPool( app->device, app->descriptor_pool, NULL );
  vkFreeMemory( app->device, app->device_memory, NULL );
  vkFreeMemory( app->device, app->host_memory, NULL );
  vkDestroyDevice( app->device, NULL );
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

  uint32_t volatile*       mem = app->mapped_memory;
  uint32_t const volatile* loc = mem + N_INTS;

  vkQueueSubmit( app->queue, 1, submit_info, fence ); // not sure when this returns?

  *mem = 1; // trigger the write
  uint64_t start = rdtscp();

  // wait for the two
  while( true ) {
    if( LIKELY( *loc == 2 ) ) break;
  }

  uint64_t finish = rdtscp();

  res = vkWaitForFences( app->device, 1, &fence, VK_TRUE, 10000000000 );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to wait for fence" );
  }

  return finish-start;
}
#endif

void
app_run( app_t* app )
{
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

  // static uint64_t tsc_freq_khz = 3892231; // AMD
  static uint64_t tsc_freq_khz = 2099944; // intel
  double          ns_per_cycle = 1./((double)(tsc_freq_khz * 1000)/1e9);
  for( size_t i = 0; i < ARRAY_SIZE( trials ); ++i ) {
    printf( "%f\n", (double)trials[i]*ns_per_cycle );
  }
#endif
}
