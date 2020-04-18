#include "app.h"
#include "common.h"
#include "log.h"
#include "option.h"

#include "volk.h"
#include <stdint.h>
#include <stdlib.h>

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

  app->device = device;
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

  VkPhysicalDevice* physical_devices = malloc( physical_device_count * sizeof( VkPhysicalDevice ) );
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

    /* Look for all of the memories that are required. */

    VkMemoryPropertyFlagBits memory_flags[3] = {
      VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,      /* need one coherent mem */
      VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,       /* one device local */
      VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT,       /* need something to transfer from */
    };

    uint32_t memory_idx[3] = { 0, 0, 0 };
    bool     found_mem[3]  = { false, false, false };

    VkPhysicalDeviceMemoryProperties mem_props[1];
    vkGetPhysicalDeviceMemoryProperties( dev, mem_props );

    uint32_t            n_mem = mem_props->memoryTypeCount;
    VkMemoryType const* mt    = mem_props->memoryTypes;
    for( uint32_t j = 0; j < n_mem; ++j ) {
      VkMemoryType mem = mt[j];

      for( size_t target = 0; target < ARRAY_SIZE( memory_idx ); ++target ) {
        if( found_mem[target] ) continue;
        if( mem.propertyFlags & memory_flags[target] ) {
          /* found one */

          memory_idx[target] = j;
          found_mem[target] = true;
        }
      }
    }

    bool found_memories = true;
    for( size_t j = 0; j < 3; ++j ) found_memories = found_memories && found_mem[j];

    if( LIKELY( found_queue && found_memories ) ) {
      LOG_INFO( "Found memories at idxs { %u, %u, %u }",
                memory_idx[0], memory_idx[1], memory_idx[2] );
      LOG_INFO( "Tranfer queue at idx %u", transfer_queue );
      open_device( app, dev, transfer_queue );
      found_device = true;
      break;
    }
    else {
      LOG_INFO( "Device %u not valid. found memory? %s found_queue %s", i,
                ( found_memories ? "YES" : "NO" ),
                ( found_queue    ? "YES" : "NO" ) );

    }

    free( props );
  }

  if( UNLIKELY( !found_device ) ) {
    FATAL( "No acceptable device found" );
  }

  free( physical_devices );
}

void
app_destroy( app_t* app )
{
  if( !app ) return;

  vkDestroyDevice( app->device, NULL );
}

void
app_run( app_t* app )
{
  
}

