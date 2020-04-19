#pragma once

#include "volk.h"

typedef struct app app_t;

struct app
{
  float      queue_priority[1];
  VkInstance instance;           // borrow
  VkDevice   device;

  /* Host/Device coherant memory of some sort */
  VkDeviceMemory coherent_memory;

  /* the above memory mapped into cpu address space */
  void*          mapped_memory;

  VkShaderModule shader;
};

void
app_init( app_t *    app,
          VkInstance instance );

void
app_destroy( app_t * app );

void
app_run( app_t * app );
