#pragma once

#include "volk.h"
// #define WOC_USE_MAILBOX

// forward decl
typedef struct GLFWwindow GLFWwindow;
typedef struct tracer     tracer_t;

typedef struct app app_t;

struct app
{
  VkInstance         instance;  // borrow
  GLFWwindow*        window;
  VkSurfaceKHR       window_surface;

  float              queue_priority[1];
  uint32_t           queue_idx;
  VkQueue            queue;
  VkDevice           device;

  VkExtent2D         swapchain_extent[1];
  VkSurfaceFormatKHR swapchain_surface_format[1];
  VkSwapchainKHR     swapchain;
  uint32_t           n_swapchain_images;
  VkImage*           swapchain_images;
  VkImageView*       image_views;

  VkShaderModule     vert;
  VkShaderModule     frag;

  VkBuffer           remote_vertex_buffer;
  VkDeviceMemory     remote_vertex_memory;

  VkBuffer           local_vertex_buffer;
  VkDeviceMemory     local_vertex_memory;
  void *             mapped_vertex_memory;

  VkPipelineLayout   pipeline_layout;
  VkRenderPass       render_pass;
  VkPipeline         graphics_pipeline;

  VkFramebuffer*     framebuffers;

  VkCommandPool      command_pool;
  VkCommandBuffer*   command_buffers;

  uint32_t           max_frames_in_flight;
  VkSemaphore*       image_avail_semaphores;
  VkSemaphore*       render_finished_semaphores;
  VkFence*           in_flight_fences;
  VkFence*           images_in_flight;

  tracer_t*          tracer;
};

void
app_init( app_t*      app,
          VkInstance  instance );

void
app_destroy( app_t* app );

void
app_run( app_t* app );
