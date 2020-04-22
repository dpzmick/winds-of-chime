#pragma once

#include "volk.h"

// forward decl
typedef struct GLFWwindow GLFWwindow;

typedef struct app app_t;

struct app
{
  VkInstance  instance;         // borrow
  GLFWwindow* window;           // borrow

  float       queue_priority;
  uint32_t    queue_idx;
  VkQueue     queue;
  VkDevice    device;

  VkDeviceMemory host_memory;
  void*          mapped_host_memory;
  VkDeviceMemory device_memory;

  VkCommandPool    cmd_pool;
  VkDescriptorPool descriptor_pool;

#if 0
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
          VkInstance  instance,
          GLFWwindow* window );

void
app_destroy( app_t* app );

void
app_run( app_t* app );
