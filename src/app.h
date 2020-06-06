#pragma once

#include "volk.h"

// forward decl
typedef struct GLFWwindow GLFWwindow;

typedef struct app app_t;

struct app
{
  VkInstance         instance;  // borrow
  GLFWwindow*        window;
  VkSurfaceKHR       window_surface;

  float    queue_priority[1];
  uint32_t queue_idx;
  VkQueue  queue;
  VkDevice device;

  VkSwapchainKHR swapchain;
  uint32_t       n_swapchain_images;
  VkImage*       swapchain_images;
  VkImageView*   image_views;

#if 0
  VkDeviceMemory host_memory;
  void*          mapped_host_memory;
  VkDeviceMemory device_memory;

  VkCommandPool    cmd_pool;
  VkDescriptorPool descriptor_pool;

  VkShaderModule shader;

  VkBuffer in_buffer;
  VkBuffer out_buffer;

  // FIXME do I need to keep these around after constructing pipeline?
  VkDescriptorSetLayout dset_layout;
  VkPipelineLayout      playout;

  VkPipeline pipeline;
  VkDescriptorSet dset;

  VkCommandBuffer cmd_buffer;
#endif
};

void
app_init( app_t*      app,
          VkInstance  instance );

void
app_destroy( app_t* app );

void
app_run( app_t* app );
