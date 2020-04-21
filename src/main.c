#include "app.h"
#include "common.h"
#include "log.h"

#include "volk.h"
#include <assert.h>
#include <dlfcn.h>
#include <stddef.h>
#include <stdlib.h>

static VkBool32
debug_callback( VkDebugUtilsMessageSeverityFlagBitsEXT      messageSeverity,
                VkDebugUtilsMessageTypeFlagsEXT             messageType,
                const VkDebugUtilsMessengerCallbackDataEXT* pCallbackData,
                void*                                       pUserData)
{
  LOG_ERROR( "Vulkan Message: %s", pCallbackData->pMessage );
  return VK_FALSE;
}

static char const * enabled_layer_names[] = {
                                             //"VK_LAYER_LUNARG_standard_validation",
};

static char const * enabled_ext_names[] = {
                                           //VK_EXT_DEBUG_UTILS_EXTENSION_NAME,
};

static VkInstanceCreateInfo instance_create_info[1] = {{
  .sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
  .pNext = NULL,
  .flags = 0,
  .pApplicationInfo = &(VkApplicationInfo){
    .sType            = VK_STRUCTURE_TYPE_APPLICATION_INFO,
    .pNext            = NULL,
    .pApplicationName = "winds of chime",
    .pEngineName      = "custom",
    .engineVersion    = 1,
    .apiVersion       = VK_MAKE_VERSION(1, 2, 0),
  },
  .enabledLayerCount       = ARRAY_SIZE( enabled_layer_names ),
  .ppEnabledLayerNames     = enabled_layer_names,
  .enabledExtensionCount   = ARRAY_SIZE( enabled_ext_names ),
  .ppEnabledExtensionNames = enabled_ext_names,
}};

static VkDebugUtilsMessengerCreateInfoEXT dbg_create_info[] = {{
  .sType           = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
  .pNext           = NULL,
  .messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT,
  .messageType     = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT,
  .pfnUserCallback = debug_callback,
  .pUserData       = NULL,
}};

int
main()
{
  VkResult                 res       = VK_SUCCESS;
  VkInstance               instance  = VK_NULL_HANDLE;
  VkDebugUtilsMessengerEXT messenger = VK_NULL_HANDLE;
  app_t                    app[1];

  res = volkInitialize();
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to initialize volk with res=%d", res );
  }

  res = vkCreateInstance( instance_create_info, NULL, &instance );
  if( UNLIKELY( res != VK_SUCCESS ) ) {
    FATAL( "Failed to create instance with res=%d", res );
  }

  volkLoadInstance( instance );

  /* res = vkCreateDebugUtilsMessengerEXT( instance, dbg_create_info, NULL, &messenger ); */
  /* if( UNLIKELY( res != VK_SUCCESS ) ) { */
  /*   FATAL( "Failed to create debug messenger ret=%d", res ); */
  /* } */

  LOG_INFO( "Created vulkan instance successfully" );

  app_init( app, instance );
  app_run( app );
  app_destroy( app );

  /* vkDestroyDebugUtilsMessengerEXT( instance, messenger, NULL ); */
  vkDestroyInstance( instance, NULL );

  return 0;
}
