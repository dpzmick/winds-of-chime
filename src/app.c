#include "app.h"
#include "common.h"
#include "log.h"

#include "volk.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MEMORY_SIZE (4ul * 1024ul * 1024ul)

static char*
read_entire_file( char const* filename,
                  size_t*     out_bytes )
{
  size_t mem_used  = 0;
  size_t mem_space = 4096;
  char*  buffer    = malloc( mem_space );
  if( !buffer ) FATAL( "Failed to allocate memory" );

  FILE* f = fopen( filename, "r" );
  if( !f ) FATAL( "Failed to open file %s", filename );

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
    if( !buffer ) FATAL( "Failed to allocate memory" );
  }
}

static void
open_device( app_t *          app,
             VkPhysicalDevice physical_device,
             uint32_t         queue_idx )
{
  app->queue_priority[0] = 1.0;

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
map_memory( void**         map_to,
            VkDevice       device,
            VkDeviceMemory memory )
{
  VkResult res = vkMapMemory( device, memory, 0, MEMORY_SIZE, 0, map_to );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to map memory" );
  }
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

    uint32_t transfer_queue = 0;
    bool     found_queue    = false;

    for( uint32_t j = 0; j < prop_cnt; ++j ) {
      VkQueueFlags flags = props[i].queueFlags;
      if( flags & VK_QUEUE_TRANSFER_BIT ) {     /* FIXME also need compute? */
        transfer_queue = j;
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
      LOG_INFO( "Tranfer queue at idx %u", transfer_queue );

      open_device( app, dev, transfer_queue );
      open_memory( &app->coherent_memory, app->device, memory_idx );
      map_memory( &app->mapped_memory, app->device, app->coherent_memory );

      /* create a shader and expose the buffer */

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
  if( shader_bytes % sizeof( uint32_t ) != 0 ) {
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
  if( res != VK_SUCCESS ) {
    FATAL( "Failed to create shader module ret=%d", res );
  }

  free( shader_contents );
}

void
app_destroy( app_t* app )
{
  if( !app ) return;

  vkDestroyShaderModule( app->device, app->shader, NULL );
  vkUnmapMemory( app->device, app->coherent_memory );
  vkFreeMemory( app->device, app->coherent_memory, NULL );
  vkDestroyDevice( app->device, NULL );
}

void
app_run( app_t* app )
{

}
