#include "app.h"
#include "common.h"
#include "util/log.h"

#include "volk.h"

#include <tracing_structs.h>

#include <GLFW/glfw3.h>
#include <assert.h>
#include <dlfcn.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

static char const * validation_layer = "VK_LAYER_KHRONOS_validation";

static VkBool32
debug_callback( VkDebugUtilsMessageSeverityFlagBitsEXT      messageSeverity,
                VkDebugUtilsMessageTypeFlagsEXT             messageType,
                const VkDebugUtilsMessengerCallbackDataEXT* pCallbackData,
                void*                                       pUserData)
{
  LOG_ERROR( "Vulkan Message: %s", pCallbackData->pMessage );
  return VK_FALSE;
}

static VkDebugUtilsMessengerCreateInfoEXT dbg_create_info[] = {{
  .sType           = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
  .pNext           = NULL,
  .messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT,
  .messageType     = // VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                     VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                   | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT,
  .pfnUserCallback = debug_callback,
  .pUserData       = NULL,
}};

static void
glfw_error_callback( int code, char const * desc )
{
  LOG_ERROR( "GLFW Error %s (%d)", desc, code );
}

static VkInstance
create_instance( VkDebugUtilsMessengerEXT* out_messenger )
{
  VkResult   vk_res   = VK_SUCCESS;
  VkInstance instance = VK_NULL_HANDLE;

  uint32_t     glfw_ext_count = 0;
  char const** glfw_exts      = glfwGetRequiredInstanceExtensions(&glfw_ext_count);

  uint32_t     enabled_layer_count = 0;
  char const** enabled_layers      = calloc( 1, sizeof( char* ) );
  if( !enabled_layers ) FATAL( "Failed to allocate memory" );

  uint32_t     enabled_ext_count = 0;
  char const** enabled_exts      = calloc( glfw_ext_count+1, sizeof( char* ) );
  if( !enabled_exts ) FATAL( "Failed to allocate memory" );

  for( uint32_t i = 0; i < glfw_ext_count; ++i ) {
    LOG_INFO( "Enabling extension %s for GLFW", glfw_exts[ i ] );
    enabled_exts[ i ] = strdup( glfw_exts[ i ] );
    enabled_ext_count += 1;
  }

  bool     do_validation    = false;
  uint32_t validation_count = 0;
  /* vk_res = vkEnumerateInstanceExtensionProperties( validation_layer, &validation_count, NULL ); */
  /* if( vk_res == VK_SUCCESS ) { */
  /*   VkExtensionProperties* props = malloc( sizeof( *props )*validation_count ); */
  /*   vk_res = vkEnumerateInstanceExtensionProperties( validation_layer, &validation_count, props ); */
  /*   if( UNLIKELY( vk_res != VK_SUCCESS ) ) FATAL( "Failed to get extensions for validation layer" ); */

  /*   for( size_t i = 0; i < validation_count; ++i ) { */
  /*     if( 0 == strcmp( props[ i ].extensionName, VK_EXT_DEBUG_UTILS_EXTENSION_NAME ) ) { */
  /*       // FIXME check */
  /*       enabled_layers[ enabled_layer_count++ ] = strdup( validation_layer ); */
  /*       enabled_exts[ enabled_ext_count++ ]     = strdup( VK_EXT_DEBUG_UTILS_EXTENSION_NAME ); */
  /*       do_validation = true; */
  /*       break; */
  /*     } */
  /*   } */
  /*   free( props ); */

  /*   if( !do_validation ) { */
  /*     LOG_INFO( "Failed to find extension for debug messaging" ); */
  /*   } */
  /* } */
  /* else { */
  /*   LOG_INFO( "Failed to get extensions for layer %s." */
  /*             " Disabling debug layers", */
  /*             validation_layer ); */
  /* } */

  VkInstanceCreateInfo instance_create_info[1] = {{
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
    .enabledLayerCount       = enabled_layer_count,
    .ppEnabledLayerNames     = enabled_layers,
    .enabledExtensionCount   = enabled_ext_count,
    .ppEnabledExtensionNames = enabled_exts,
  }};

  vk_res = vkCreateInstance( instance_create_info, NULL, &instance );
  if( UNLIKELY( vk_res != VK_SUCCESS ) ) {
    FATAL( "Failed to create instance with res=%d", vk_res );
  }

  volkLoadInstance( instance );

  if( do_validation ) {
    LOG_INFO( "Installing debug messenger" );
    vk_res = vkCreateDebugUtilsMessengerEXT( instance, dbg_create_info, NULL, out_messenger );
    if( UNLIKELY( vk_res != VK_SUCCESS ) ) {
      FATAL( "Failed to create debug messenger ret=%d", vk_res );
    }
  }
  else {
    *out_messenger = VK_NULL_HANDLE;
  }

  LOG_INFO( "Created vulkan instance successfully" );

  for( size_t i = 0; i < enabled_ext_count; ++i ) free( (void*)enabled_exts[ i ] );
  free( enabled_exts );

  for( size_t i = 0; i < enabled_layer_count; ++i ) free( (void*)enabled_layers[ i ] );
  free( enabled_layers );

  return instance;
}

int
main()
{
  VkResult                 vk_res    = VK_SUCCESS;
  int                      glfw_res  = 0;
  VkInstance               instance  = VK_NULL_HANDLE;
  VkDebugUtilsMessengerEXT messenger = VK_NULL_HANDLE;
  app_t                    app[1];

  glfwSetErrorCallback( glfw_error_callback );

  glfw_res = glfwInit();
  if( UNLIKELY( glfw_res != GLFW_TRUE ) ) {
    FATAL( "Failed to initialize glfw with %d", glfw_res );
  }

  vk_res = volkInitialize();
  if( UNLIKELY( vk_res != VK_SUCCESS ) ) {
    FATAL( "Failed to initialize volk with res=%d", vk_res );
  }

  instance = create_instance( &messenger ); // crashes on failure

  app_init( app, instance );
  app_run( app );
  app_destroy( app );

  if( messenger != VK_NULL_HANDLE ) {
    vkDestroyDebugUtilsMessengerEXT( instance, messenger, NULL );
  }
  vkDestroyInstance( instance, NULL );

  glfwTerminate();
  return 0;
}
