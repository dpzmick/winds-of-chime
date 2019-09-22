#define GLFW_INCLUDE_VULKAN // some special glfw junk
#include <GLFW/glfw3.h>

#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void init_app_info(VkApplicationInfo* info)
{
  memset(info, 0, sizeof(*info));
  info->sType              = VK_STRUCTURE_TYPE_APPLICATION_INFO;
  info->pApplicationName   = "Hello Triangle";
  info->applicationVersion = VK_MAKE_VERSION(1, 0, 0);
  info->pEngineName        = "No engine";
  info->engineVersion      = VK_MAKE_VERSION(1, 0, 0);
  info->apiVersion         = VK_API_VERSION_1_0;
}

static void init_create_info(VkInstanceCreateInfo*    info,
                             VkApplicationInfo const* app_info)
{
  uint32_t     ext_cnt = 0;
  char const** exts    = NULL;

  exts = glfwGetRequiredInstanceExtensions(&ext_cnt);
  // FIXME errors?

  memset(info, 0, sizeof(*info));
  info->sType                   = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; // FIXME is this some sort of tagged union?
  info->pApplicationInfo        = app_info; // must not move
  info->enabledExtensionCount   = ext_cnt;
  info->ppEnabledExtensionNames = exts;
  info->enabledLayerCount       = 0; // validation layers, opt-in support for UB checks, misuse checks, etc
}

static uint32_t select_device(VkPhysicalDevice* devs,
                              uint32_t          n_dev)
{
  // just pick the first one that has the feautes we need
  // which are, swap chain support, glfw extensions
}

int main()
{
  // apparently glfw is nasty and has a bunch of global state
  // sdl seems to be better in this regard
  glfwInit();

  glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
  glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);   // apparently problematic?
  glfwWindowHint(GLFW_FLOATING, GLFW_TRUE);     // try to cooperate xmonad

  GLFWwindow* win = glfwCreateWindow(800, 600, "Vulkan window", NULL, NULL);
  if (!win) abort();

  VkApplicationInfo app_info[1];
  init_app_info(app_info);

  VkInstanceCreateInfo create_info[1];
  init_create_info(create_info, app_info);

  VkInstance instance = VK_NULL_HANDLE;
  VkResult result = vkCreateInstance(create_info, NULL, &instance);
  if (result != VK_SUCCESS) abort();

  VkSurfaceKHR surface;
  if (VK_SUCCESS != glfwCreateWindowSurface(instance, win, NULL, &surface)) abort();

  // FIXME might be worth while supporting the validation layers and debug mode
  // jazz

  VkPhysicalDevice phy = VK_NULL_HANDLE;
  uint32_t dev_cnt = 0;
  vkEnumeratePhysicalDevices(instance, &dev_cnt, NULL);
  if (!dev_cnt) abort();
  printf("dev_cnt: %d\n", dev_cnt);

  VkPhysicalDevice* phys = malloc(dev_cnt * sizeof(*phys));
  vkEnumeratePhysicalDevices(instance, &dev_cnt, phys);
  phy = phys[0]; // FIXME is there a good way to read only the first device?

  // get the important queues
  uint32_t graphics_queue_idx = UINT32_MAX;
  uint32_t present_queue_idx  = UINT32_MAX;

  uint32_t queue_cnt = 0;
  vkGetPhysicalDeviceQueueFamilyProperties(phy, &queue_cnt, NULL);
  VkQueueFamilyProperties* qs = malloc(queue_cnt * sizeof(*qs));
  if (!qs) abort();
  vkGetPhysicalDeviceQueueFamilyProperties(phy, &queue_cnt, qs);
  for (size_t i = 0; i < queue_cnt; ++i) {
    if (!qs[i].queueCount) continue;
    if (qs[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) graphics_queue_idx = i;

    VkBool32 presentSupport = false;
    vkGetPhysicalDeviceSurfaceSupportKHR(phy, i, surface, &presentSupport);
    if (presentSupport) present_queue_idx = i;
  }

  if (graphics_queue_idx == UINT32_MAX) abort();
  if (present_queue_idx == UINT32_MAX)  abort();

  // bind to device
  VkDevice device;

  float prio = 1.0f;
  VkDeviceQueueCreateInfo q_create_info[2];
  memset(q_create_info, 0, sizeof(q_create_info));
  q_create_info[0].sType            = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
  q_create_info[0].queueFamilyIndex = graphics_queue_idx;
  q_create_info[0].queueCount       = 1;
  q_create_info[0].pQueuePriorities = &prio; // possible array of sz queueCount?

  q_create_info[1].sType            = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
  q_create_info[1].queueFamilyIndex = present_queue_idx;
  q_create_info[1].queueCount       = 1;
  q_create_info[1].pQueuePriorities = &prio; // possible array of sz queueCount?

  VkPhysicalDeviceFeatures dev_features[1]; // not doing anything interestng with this
  memset(dev_features, 0, sizeof(dev_features));

  char const* dev_ext[] = {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME,
  };

  VkDeviceCreateInfo dev_create_info[1];
  memset(dev_create_info, 0, sizeof(dev_create_info));
  dev_create_info->sType                   = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
  dev_create_info->pQueueCreateInfos       = q_create_info;
  dev_create_info->queueCreateInfoCount    = 2;
  dev_create_info->pEnabledFeatures        = dev_features;
  dev_create_info->enabledExtensionCount   = 0; // FIXME need glfw?
  dev_create_info->ppEnabledExtensionNames = dev_ext;
  dev_create_info->enabledLayerCount       = 1;

  if (VK_SUCCESS != vkCreateDevice(phy, dev_create_info, NULL, &device)) abort();

  VkQueue graphics_queue; // owned by device
  vkGetDeviceQueue(device, graphics_queue_idx, 0, &graphics_queue);

  VkQueue present_queue; // owned by device
  vkGetDeviceQueue(device, present_queue_idx, 0, &present_queue);

  while (!glfwWindowShouldClose(win)) {
    glfwPollEvents();
  }

  vkDestroyDevice(device, NULL);
  vkDestroySurfaceKHR(instance, surface, NULL);
  vkDestroyInstance(instance, NULL);
  glfwDestroyWindow(win);
  glfwTerminate();
}
