#include "app.h"
#include "common.h"
#include "log.h"

#include "volk.h"
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
open_device( app_t *          app,
             VkPhysicalDevice physical_device,
             uint32_t         queue_idx )
{
  app->queue_priority[0] = 1.0;
  app->compute_queue_idx = queue_idx;

  const VkDeviceQueueCreateInfo q_create[] = {{
      .sType            = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
      .pNext            = NULL,
      .flags            = 0,
      .queueFamilyIndex = queue_idx,
      .queueCount       = 1,
      .pQueuePriorities = app->queue_priority,
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
             uint32_t        memory_type_idx )
{
  VkMemoryAllocateInfo info[] = {{
      .sType           = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
      .pNext           = NULL,
      .allocationSize  = MEMORY_SIZE,
      .memoryTypeIndex = memory_type_idx,
  }};

  VkResult res = vkAllocateMemory( device, info, NULL, memory );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to allocate memory on device, ret=%d", res );
  }
}

static void
map_memory( void volatile** map_to,
            VkDevice        device,
            VkDeviceMemory  memory )
{
  VkResult res = vkMapMemory( device, memory, 0, MEMORY_SIZE, 0, (void**)map_to );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to map memory" );
  }

  /* // FIXME want this to be closer to the actual loop, testing */
  /* // coherance latency after all */
  /* size_t n = (2*1024*1024)/sizeof(uint32_t); */
  /* uint32_t volatile* into = *map_to; */
  /* for( size_t i = 0; i < n; ++i ) { */
  /*   into[i] = (uint32_t)i; */
  /* } */
}

void
app_init( app_t*     app,
          VkInstance instance )
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

  bool found_device = false;
  for( uint32_t i = 0; i < physical_device_count; ++i ) {
    VkPhysicalDevice dev = physical_devices[i];

    VkQueueFamilyProperties* props    = NULL;
    uint32_t                 prop_cnt = 0;

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, NULL );
    props = malloc( sizeof( *props ) * prop_cnt );
    if( !props ) FATAL( "Failed to allocate memory" );

    vkGetPhysicalDeviceQueueFamilyProperties( dev, &prop_cnt, props );

    uint32_t compute_queue = 0;
    bool     found_queue   = false;

    for( uint32_t j = 0; j < prop_cnt; ++j ) {
      VkQueueFlags flags = props[i].queueFlags;
      if( flags & VK_QUEUE_COMPUTE_BIT ) {
        compute_queue = j;
        found_queue = true;
        break;
      }
    }

    uint32_t memory_idx = 0;
    bool     found_mem  = false;

    VkPhysicalDeviceMemoryProperties mem_props[1];
    vkGetPhysicalDeviceMemoryProperties( dev, mem_props );

    uint32_t            n_mem = mem_props->memoryTypeCount;
    VkMemoryType const* mt    = mem_props->memoryTypes;
    for( uint32_t j = 0; j < n_mem; ++j ) {
      if( mt[j].propertyFlags & VK_MEMORY_PROPERTY_HOST_COHERENT_BIT ) {
        memory_idx = j;
        found_mem = true;
        break;
      }
    }

    if( LIKELY( found_queue && found_mem ) ) {
      LOG_INFO( "Found memory at idx %u", memory_idx );
      LOG_INFO( "Compute queue at idx %u", compute_queue );

      open_device( app, dev, compute_queue );
      open_memory( &app->coherent_memory, app->device, memory_idx );
      map_memory( &app->mapped_memory, app->device, app->coherent_memory );

      found_device = true;
      break;
    }
    else {
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
      .buffer = app->in_buffer,
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
      .flags            = 0,
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

  VkCommandBufferBeginInfo cbbi[] = {{
      .sType            = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      .pNext            = NULL,
      .flags            = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      .pInheritanceInfo = NULL,
  }};

  /* Begin recording the command buffer */
  res = vkBeginCommandBuffer( app->cmd_buffer, cbbi );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to begin command buffer" );
  }

  vkCmdBindPipeline( app->cmd_buffer, VK_PIPELINE_BIND_POINT_COMPUTE, app->pipeline );
  vkCmdBindDescriptorSets( app->cmd_buffer, VK_PIPELINE_BIND_POINT_COMPUTE, app->playout, 0, 1, &app->dset, 0, NULL );
  vkCmdDispatch( app->cmd_buffer, N_INTS, 1, 1 );

  res = vkEndCommandBuffer( app->cmd_buffer );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to end command buffer" );
  }

  // sizes given to the shader, how do I keep these in sync?
  // overall, vulkan doesn't seem to be like a great way to do compute work..
}

void
app_destroy( app_t* app )
{
  if( !app ) return;

  /* cmd_buffer? */
  vkDestroyCommandPool( app->device, app->cmd_pool, NULL );
  /* dset? */
  vkDestroyDescriptorPool( app->device, app->pool, NULL );
  vkDestroyPipeline( app->device, app->pipeline, NULL );
  vkDestroyPipelineLayout( app->device, app->playout, NULL );
  vkDestroyDescriptorSetLayout( app->device, app->dset_layout, NULL );
  vkDestroyBuffer( app->device, app->in_buffer, NULL );
  vkDestroyBuffer( app->device, app->out_buffer, NULL );
  vkDestroyShaderModule( app->device, app->shader, NULL );
  vkUnmapMemory( app->device, app->coherent_memory );
  vkFreeMemory( app->device, app->coherent_memory, NULL );
  vkDestroyDevice( app->device, NULL );
}

static uint64_t
rdtscp( void )
{
  uint32_t hi, lo;
  __asm__ volatile( "rdtscp": "=a"(lo), "=d"(hi));
  return (uint64_t)lo | ( (uint64_t)hi << 32 );
}

void
app_run( app_t* app )
{
  // submit the command once
  VkSubmitInfo submit_info[] = {{
      .sType              = VK_STRUCTURE_TYPE_SUBMIT_INFO,
      .pNext              = NULL,
      .commandBufferCount = 1,
      .pCommandBuffers    = &app->cmd_buffer,
  }};

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

  // *(uint8_t*)(app->mapped_memory) = 9;

  uint64_t start = rdtscp();

  vkQueueSubmit( app->queue, 1, submit_info, fence );

  /* while( true ) { */
  /*   uint8_t const volatile* loc = (uint8_t const*)(app->mapped_memory) + 2ul*1024ul*1024ul; */
  /*   if( *loc == 9 ) break; */
  /* } */

  res = vkWaitForFences( app->device, 1, &fence, VK_TRUE, 10000000000 );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to wait for fence" );
  }

  uint64_t finish = rdtscp();
  LOG_INFO( "start=%zu finish=%zu, dt=%zu", start, finish, finish-start );

  uint32_t volatile* out = (uint32_t volatile*)((char*)app->mapped_memory + 2ul*1024ul*1024ul);
  for( size_t i = 0; i < N_INTS; ++i ) {
    LOG_INFO( "[%zu] = %u", i, out[i] );
  }

  vkDestroyFence( app->device, fence, NULL );
}
