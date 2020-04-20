#pragma once

#include "volk.h"

typedef struct app app_t;

struct app
{
  float      queue_priority[1];
  VkInstance instance;          // borrow
  VkDevice   device;
  uint32_t   compute_queue_idx;
  VkQueue    queue;

  /* Host/Device coherant memory of some sort */
  VkDeviceMemory coherent_memory;

  /* the above memory mapped into cpu address space */
  void volatile* mapped_memory;

  VkShaderModule shader;

  VkBuffer in_buffer;
  VkBuffer out_buffer;

  // FIXME do I need to keep these around after constructing pipeline?
  VkDescriptorSetLayout dset_layout;
  VkPipelineLayout      playout;

  VkPipeline pipeline;
  VkDescriptorPool pool;
  VkDescriptorSet dset;

  VkCommandPool   cmd_pool;
  VkCommandBuffer cmd_buffer;
};

void
app_init( app_t *    app,
          VkInstance instance );

void
app_destroy( app_t * app );

void
app_run( app_t * app );
