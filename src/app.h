#pragma once

#include "volk.h"

typedef struct app app_t;

struct app
{
  VkInstance instance;           // borrow
  VkDevice   device;             // owned
  float      queue_priority[1];
};

void
app_init( app_t *    app,
          VkInstance instance );

void
app_destroy( app_t * app );

void
app_run( app_t * app );
