#define GLFW_INCLUDE_VULKAN // some special glfw junk
#include <GLFW/glfw3.h>
#include <vulkan/vulkan_core.h>

#include <assert.h>
#include <errno.h>
#include <execinfo.h>
#include <stdbool.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

#define MIN(a,b) ((a) < (b) ? (a) : (b))
#define MAX(a,b) ((a) >= (b) ? (a) : (b))

typedef struct { // chosen settings
  uint32_t                      min_image_count;
  VkSurfaceFormatKHR            surface_format[1];
  VkPresentModeKHR              present_mode[1];
  VkExtent2D                    extent[1];
  VkSurfaceTransformFlagBitsKHR current_transform[1];
} sc_cap_t;

char const* read_file(char const* fn, size_t* n)
{
  size_t rem = 4096;
  char* b = malloc(rem);
  if (!b) abort();

  int fd = open(fn, O_RDONLY);
  if (fd < 0) abort();

  *n = 0;
  size_t r = 0;
  char* ptr = b;
  while (1) {
    errno = 0;
    ssize_t cnt = read(fd, ptr, rem);
    if (cnt == 0 && errno == 0) break;
    if (cnt < 0) abort();
    if (cnt == 0) abort();
    ptr += cnt;
    *n += (size_t)cnt;
    rem -= cnt;
    if (rem == 0) abort();
  }
  close(fd);
  return b;
}

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

static VKAPI_ATTR VkBool32 VKAPI_CALL
debugCallback(VkDebugUtilsMessageSeverityFlagBitsEXT      severity,
              VkDebugUtilsMessageTypeFlagsEXT             type,
              const VkDebugUtilsMessengerCallbackDataEXT* pCallbackData,
              void*                                       pUserData)
{
  fprintf(stderr, "%s\n\n", pCallbackData->pMessage);
  return VK_FALSE;
}

static void init_create_info(VkInstanceCreateInfo*    info,
                             VkApplicationInfo const* app_info)
{
  uint32_t     ext_cnt   = 0;
  char const** exts      = malloc(16*sizeof(*exts));
  uint32_t     layer_cnt = 1;
  char const** layers    = malloc(16*sizeof(*layers));
  layers[0] = "VK_LAYER_LUNARG_standard_validation";

  char const** exts_glfw = glfwGetRequiredInstanceExtensions(&ext_cnt);

  for (size_t i = 0; i < ext_cnt; ++i) {
    exts[i] = exts_glfw[i];
  }

  exts[ext_cnt] = VK_EXT_DEBUG_UTILS_EXTENSION_NAME;
  ext_cnt += 1;

  memset(info, 0, sizeof(*info));
  info->sType                   = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
  info->pApplicationInfo        = app_info; // must not move
  info->enabledExtensionCount   = ext_cnt;
  info->ppEnabledExtensionNames = exts;
  info->enabledLayerCount       = layer_cnt;
  info->ppEnabledLayerNames     = layers;
}

static sc_cap_t select_caps(VkPhysicalDevice phy,
                            VkSurfaceKHR     surface,
                            sc_cap_t*        ret)
{
  VkSurfaceCapabilitiesKHR caps[1];
  vkGetPhysicalDeviceSurfaceCapabilitiesKHR(phy, surface, caps);

  // just taking the "default" for each
  uint32_t cnt = 1;
  vkGetPhysicalDeviceSurfaceFormatsKHR(phy, surface, &cnt, ret->surface_format);
  vkGetPhysicalDeviceSurfacePresentModesKHR(phy, surface, &cnt, ret->present_mode);

  if (caps->currentExtent.width != UINT32_MAX) {
    *(ret->extent) = caps->currentExtent;
  }
  else {
    VkExtent2D actual = {800, 600}; // why do I have to do this?
    *(ret->extent) = actual; // FIXME clip?
  }

  *(ret->current_transform) = caps->currentTransform;
}

int main()
{
  // apparently glfw is nasty and has a bunch of global state
  // sdl seems to be better in this regard
  int code = glfwInit();
  if (code != GLFW_TRUE) {
    fprintf(stderr, "Failed to init glfw with %x\n", glfwGetError(NULL));
    return 1;
  }

  glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
  glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);   // apparently problematic?
  glfwWindowHint(GLFW_FLOATING, GLFW_TRUE);     // try to cooperate xmonad

  GLFWwindow* win = glfwCreateWindow(800, 600, "Vulkan window", NULL, NULL);
  if (!win) {
    int code = glfwGetError(NULL);
    fprintf(stderr, "Failed to create glfw window with %x\n", code);
    return 1;
  }

  VkApplicationInfo app_info[1];
  init_app_info(app_info);

  VkInstanceCreateInfo create_info[1];
  init_create_info(create_info, app_info);

  VkInstance instance = VK_NULL_HANDLE;
  VkResult result = vkCreateInstance(create_info, NULL, &instance);
  if (result != VK_SUCCESS) {
    fprintf(stderr, "Failed with %d\n", result);
    fprintf(stderr, "eq %d\n", VK_ERROR_LAYER_NOT_PRESENT == result);
    return 1;
  }

  VkDebugUtilsMessengerCreateInfoEXT dbg_create_info[1];
  memset(dbg_create_info, 0, sizeof(dbg_create_info));
  dbg_create_info->sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
  dbg_create_info->messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                                   | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                                   | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;
  dbg_create_info->messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                               | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                               | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
  dbg_create_info->pfnUserCallback = debugCallback;
  dbg_create_info->pUserData = NULL;

  VkDebugUtilsMessengerEXT messenger;
  PFN_vkCreateDebugUtilsMessengerEXT fp = (PFN_vkCreateDebugUtilsMessengerEXT)vkGetInstanceProcAddr(instance, "vkCreateDebugUtilsMessengerEXT");
  if (!fp) abort();
  fp(instance, dbg_create_info, NULL, &messenger);

  VkSurfaceKHR surface;
  if (VK_SUCCESS != glfwCreateWindowSurface(instance, win, NULL, &surface)) abort();

  uint32_t xxx = 0;
  vkEnumeratePhysicalDevices(instance, &xxx, NULL); xxx = 1; // make logging stop

  VkPhysicalDevice phy = VK_NULL_HANDLE;
  vkEnumeratePhysicalDevices(instance, &xxx, &phy);

  // get the important queues
  uint32_t graphics_queue_idx = UINT32_MAX;
  uint32_t present_queue_idx  = UINT32_MAX;

  uint32_t queue_cnt = 32;
  vkGetPhysicalDeviceQueueFamilyProperties(phy, &queue_cnt, NULL); // logging
  VkQueueFamilyProperties* qs = malloc(128*sizeof(*qs));
  vkGetPhysicalDeviceQueueFamilyProperties(phy, &queue_cnt, qs);
  printf("queue_cnt: %d\n", queue_cnt);
  for (size_t i = 0; i < queue_cnt; ++i) {
    if (!qs[i].queueCount) continue;
    if (qs[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) graphics_queue_idx = i;

    VkBool32 presentSupport = false;
    vkGetPhysicalDeviceSurfaceSupportKHR(phy, i, surface, &presentSupport);
    if (presentSupport) present_queue_idx = i;
  }

  printf("graphics: %u present: %u\n", graphics_queue_idx, present_queue_idx);
  if (graphics_queue_idx == UINT32_MAX) abort();
  if (present_queue_idx == UINT32_MAX)  abort();

  // bind to device
  VkDevice device;

  // one per uniq queue family (in this case, there's one only)
  float prio = 1.0f;
  VkDeviceQueueCreateInfo q_create_info[1];
  memset(q_create_info, 0, sizeof(q_create_info));
  q_create_info[0].sType            = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
  q_create_info[0].queueFamilyIndex = graphics_queue_idx;
  q_create_info[0].queueCount       = 1;
  q_create_info[0].pQueuePriorities = &prio; // possible array of sz queueCount?

  VkPhysicalDeviceFeatures dev_features[1]; // not doing anything interestng with this
  memset(dev_features, 0, sizeof(dev_features));

  char const* dev_ext[] = {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME,
  };

  char const* dev_layers[] = {
    "VK_LAYER_LUNARG_standard_validation",
  };

  VkDeviceCreateInfo dev_create_info[1];
  memset(dev_create_info, 0, sizeof(dev_create_info));
  dev_create_info->sType                   = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
  dev_create_info->pQueueCreateInfos       = q_create_info;
  dev_create_info->queueCreateInfoCount    = 1;
  dev_create_info->pEnabledFeatures        = dev_features;
  dev_create_info->enabledExtensionCount   = 1;
  dev_create_info->ppEnabledExtensionNames = dev_ext;
  dev_create_info->enabledLayerCount       = 0;
  dev_create_info->ppEnabledLayerNames     = dev_layers;
  dev_create_info->enabledLayerCount       = 1;

  if (VK_SUCCESS != vkCreateDevice(phy, dev_create_info, NULL, &device)) abort();

  VkQueue graphics_queue; // owned by device
  vkGetDeviceQueue(device, graphics_queue_idx, 0, &graphics_queue);

  VkQueue present_queue; // owned by device
  vkGetDeviceQueue(device, present_queue_idx, 0, &present_queue);

  uint32_t indices[] = {graphics_queue_idx, present_queue_idx};
  sc_cap_t caps[1];
  select_caps(phy, surface, caps);

  VkSwapchainCreateInfoKHR sc_create_info[1];
  memset(sc_create_info, 0, sizeof(sc_create_info));
  sc_create_info->sType            = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
  sc_create_info->surface          = surface;
  sc_create_info->minImageCount    = 4 + 1;
  sc_create_info->imageFormat      = caps->surface_format->format;
  sc_create_info->imageColorSpace  = caps->surface_format->colorSpace;
  sc_create_info->compositeAlpha   = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
  sc_create_info->presentMode      = *(caps->present_mode);
  sc_create_info->imageExtent      = *(caps->extent);
  sc_create_info->imageArrayLayers = 1;
  sc_create_info->imageUsage       = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT; // how are these used
  if (graphics_queue_idx == present_queue_idx) {
    sc_create_info->imageSharingMode      = VK_SHARING_MODE_EXCLUSIVE;
    sc_create_info->queueFamilyIndexCount = 0;
    sc_create_info->pQueueFamilyIndices   = 0;
  }
  else {
    sc_create_info->imageSharingMode      = VK_SHARING_MODE_CONCURRENT;
    sc_create_info->queueFamilyIndexCount = 2;
    sc_create_info->pQueueFamilyIndices   = indices;
  }
  sc_create_info->preTransform     = *(caps->current_transform);
  sc_create_info->clipped          = VK_TRUE;
  sc_create_info->oldSwapchain     = VK_NULL_HANDLE;

  // wow that's too much
  VkSwapchainKHR swapchain;
  if (VK_SUCCESS != vkCreateSwapchainKHR(device, sc_create_info, NULL, &swapchain)) {
    abort();
  }

  uint32_t n_image;
  vkGetSwapchainImagesKHR(device, swapchain, &n_image, NULL);
  VkImage* images = malloc(n_image  * sizeof(*images));
  vkGetSwapchainImagesKHR(device, swapchain, &n_image, images);

  VkImageView* views = malloc(n_image * sizeof(*views));
  for (size_t i = 0; i < n_image; ++i) {
    VkImageViewCreateInfo info[1];
    memset(info, 0, sizeof(info));
    info->sType                           = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    info->image                           = images[i];
    info->viewType                        = VK_IMAGE_VIEW_TYPE_2D;
    info->format                          = caps->surface_format->format;
    info->components.r                    = VK_COMPONENT_SWIZZLE_IDENTITY;
    info->components.g                    = VK_COMPONENT_SWIZZLE_IDENTITY;
    info->components.b                    = VK_COMPONENT_SWIZZLE_IDENTITY;
    info->components.a                    = VK_COMPONENT_SWIZZLE_IDENTITY;
    info->subresourceRange.aspectMask     = VK_IMAGE_ASPECT_COLOR_BIT;
    info->subresourceRange.baseMipLevel   = 0;
    info->subresourceRange.levelCount     = 1;
    info->subresourceRange.baseArrayLayer = 0;
    info->subresourceRange.layerCount     = 1;

    if (VK_SUCCESS != vkCreateImageView(device, info, NULL, views + i)) abort();
  }

  // last but not least, graphics pipeline, again, whatever that is
  // apparently a fairly standard term in graphics? All I want to do is draw a
  // triangle!!!
  // is this somehow related to the way the hardware is designed?

  // vertex shader: {incoming vertex} -> {clip_coordinate}
  // - takes the incoming vertex and any specified properties
  // - produces clip coordinates and anything that needs to go to fragment
  //   shader
  //
  // vocab: clip coordinate = 4 tuple, last element divides the other two to
  // convert to device coordinates (how is device coord 3d?)

  size_t n = 0;
  char const* vert = read_file("shaders/shader.vert.spv", &n);
  VkShaderModuleCreateInfo shader_create_info[1];
  memset(shader_create_info, 0, sizeof(shader_create_info));
  shader_create_info->sType    = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
  shader_create_info->codeSize = n;
  shader_create_info->pCode    = (uint32_t const*)vert;

  VkShaderModule vert_shader;
  if (VK_SUCCESS != vkCreateShaderModule(device, shader_create_info, NULL, &vert_shader)) abort();
  free((void*)vert);

  n = 0;
  char const* frag = read_file("shaders/shader.frag.spv", &n);
  memset(shader_create_info, 0, sizeof(shader_create_info));
  shader_create_info->sType    = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
  shader_create_info->codeSize = n;
  shader_create_info->pCode    = (uint32_t const*)frag;

  VkShaderModule frag_shader;
  if (VK_SUCCESS != vkCreateShaderModule(device, shader_create_info, NULL, &frag_shader)) abort();
  free((void*)frag);

  VkPipelineShaderStageCreateInfo pipe1[1];
  memset(pipe1, 0, sizeof(pipe1));
  pipe1->sType  = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
  pipe1->stage  = VK_SHADER_STAGE_VERTEX_BIT;
  pipe1->module = vert_shader;
  pipe1->pName  = "main";

  VkPipelineShaderStageCreateInfo pipe2[1];
  memset(pipe2, 0, sizeof(pipe2));
  pipe2->sType  = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
  pipe2->stage  = VK_SHADER_STAGE_FRAGMENT_BIT;
  pipe2->module = frag_shader;
  pipe2->pName  = "main";

  VkPipelineShaderStageCreateInfo stages[] = { *pipe1, *pipe2 };

  VkPipelineVertexInputStateCreateInfo vii[1];
  memset(vii, 0, sizeof(vii));
  vii->sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
  vii->vertexBindingDescriptionCount = 0;
  vii->pVertexBindingDescriptions = NULL;
  vii->vertexAttributeDescriptionCount = 0;
  vii->pVertexAttributeDescriptions = NULL;

  // end it now please
  VkPipelineInputAssemblyStateCreateInfo pasci[1];
  memset(pasci, 0, sizeof(pasci));
  pasci->sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
  pasci->topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
  pasci->primitiveRestartEnable = VK_FALSE;

  // WTF is a scissors
  VkViewport viewport;
  memset(&viewport, 0, sizeof(viewport));
  viewport.x = 0.0f;
  viewport.y = 0.0f;
  viewport.width = (float)caps->extent->width;
  viewport.height = (float)caps->extent->height;
  viewport.minDepth = 0.0f;
  viewport.maxDepth = 1.0f;

  VkRect2D scissor;
  memset(&scissor, 0, sizeof(scissor));
  // offset 0, 0
  scissor.extent = *(caps->extent);

  VkPipelineViewportStateCreateInfo viewportState[1];
  memset(viewportState, 0, sizeof(viewportState));
  viewportState->sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO;
  viewportState->viewportCount = 1;
  viewportState->pViewports = &viewport;
  viewportState->scissorCount = 1;
  viewportState->pScissors = &scissor;

  VkPipelineRasterizationStateCreateInfo rast[1];
  memset(rast, 0, sizeof(rast));
  rast->sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
  rast->depthClampEnable = VK_FALSE;
  rast->rasterizerDiscardEnable = VK_FALSE;
  rast->polygonMode = VK_POLYGON_MODE_FILL;
  rast->lineWidth = 1.0f;
  rast->cullMode = VK_CULL_MODE_BACK_BIT;
  rast->frontFace = VK_FRONT_FACE_CLOCKWISE;
  rast->depthBiasEnable = VK_FALSE;
  rast->depthBiasConstantFactor = 0.0f;
  rast->depthBiasClamp = 0.0f;
  rast->depthBiasSlopeFactor = 0.0f;

  VkPipelineMultisampleStateCreateInfo msaa[1];
  memset(msaa, 0, sizeof(msaa));
  msaa->sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
  msaa->sampleShadingEnable = VK_FALSE;
  msaa->rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;
  msaa->minSampleShading = 1.0f;
  msaa->pSampleMask = NULL;
  msaa->alphaToCoverageEnable = VK_FALSE;
  msaa->alphaToOneEnable = VK_FALSE;

  VkPipelineColorBlendAttachmentState blend[1];
  memset(blend, 0, sizeof(blend));
  blend->colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
  blend->blendEnable = VK_FALSE;
  // bunch of optional fields
  // some other thing I'm also ignoring

  VkPipelineColorBlendStateCreateInfo cblend[1];
  memset(cblend, 0, sizeof(cblend));
  cblend->sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
  cblend->logicOpEnable = VK_FALSE;
  cblend->logicOp = VK_LOGIC_OP_COPY;
  cblend->attachmentCount = 1;
  cblend->pAttachments = blend;
  // some optional stuff

  VkDynamicState dynamic_states[] = {
    VK_DYNAMIC_STATE_VIEWPORT,
    VK_DYNAMIC_STATE_LINE_WIDTH,
  };

  VkPipelineDynamicStateCreateInfo dynamic_state[1];
  memset(dynamic_state, 0, sizeof(dynamic_state));
  dynamic_state->sType = VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO;
  dynamic_state->dynamicStateCount = 2;
  dynamic_state->pDynamicStates = dynamic_states;

  VkPipelineLayoutCreateInfo pipe_create[1];
  memset(pipe_create, 0, sizeof(pipe_create));
  pipe_create->sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
  pipe_create->setLayoutCount = 0;
  pipe_create->pSetLayouts = NULL;
  pipe_create->pushConstantRangeCount = 0;
  pipe_create->pPushConstantRanges = NULL;

  VkPipelineLayout pipeline_layout;
  if (VK_SUCCESS != vkCreatePipelineLayout(device, pipe_create, NULL, &pipeline_layout)) abort();

  // oh god there's more
  // we need to tell vulkan about the framebuffer attachmenets that will be used
  // while rendering.
  VkAttachmentDescription colorAttach;
  memset(&colorAttach, 0, sizeof(colorAttach));
  colorAttach.format = caps->surface_format->format;
  colorAttach.samples = VK_SAMPLE_COUNT_1_BIT;
  colorAttach.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
  colorAttach.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
  colorAttach.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
  colorAttach.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
  colorAttach.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
  colorAttach.finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;

  // how can there possibly be more

  VkAttachmentReference caref[1];
  memset(caref, 0, sizeof(caref));
  caref->attachment = 0;
  caref->layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;

  VkSubpassDescription subpass[1];
  memset(subpass, 0, sizeof(subpass));
  subpass->pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
  subpass->colorAttachmentCount = 1;
  subpass->pColorAttachments = caref;

  VkRenderPassCreateInfo rpci[1];
  memset(rpci, 0, sizeof(rpci));
  rpci->sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
  rpci->attachmentCount = 1;
  rpci->pAttachments = &colorAttach;
  rpci->subpassCount = 1;
  rpci->pSubpasses = subpass;

  VkRenderPass renderPass;
  if (VK_SUCCESS != vkCreateRenderPass(device, rpci, NULL, &renderPass)) abort();

  VkGraphicsPipelineCreateInfo gpci[1];
  memset(gpci, 0, sizeof(gpci));
  gpci->sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
  gpci->stageCount = 2;
  gpci->pStages = stages;
  gpci->pVertexInputState = vii;
  gpci->pInputAssemblyState = pasci;
  gpci->pViewportState = viewportState;
  gpci->pRasterizationState = rast;
  gpci->pMultisampleState = msaa;
  gpci->pColorBlendState = cblend;
  gpci->layout = pipeline_layout;
  gpci->renderPass = renderPass;
  gpci->subpass = 0;

  VkPipeline graphicsPipeline;
  if (VK_SUCCESS != vkCreateGraphicsPipelines(device, VK_NULL_HANDLE, 1, gpci, NULL, &graphicsPipeline)) abort();

  // now we somehow need even more
  VkFramebuffer* fbs = malloc(n_image * sizeof(*fbs));
  if (!fbs) abort();
  for (size_t i = 0; i < n_image; ++i) {
    VkFramebufferCreateInfo c[1];
    memset(c, 0, sizeof(c));
    c->sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
    c->renderPass = renderPass;
    c->attachmentCount = 1;
    c->pAttachments = views + i;
    c->width = caps->extent->width;
    c->height = caps->extent->height;
    c->layers = 1;

    if (VK_SUCCESS != vkCreateFramebuffer(device, c, NULL, fbs + i)) abort();
  }

  // here comes the good part, I guess?

  VkCommandPoolCreateInfo pci[1];
  memset(pci, 0, sizeof(pci));
  pci->sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
  pci->queueFamilyIndex = graphics_queue_idx;
  pci->flags = 0;

  VkCommandPool commandPool;
  if (VK_SUCCESS != vkCreateCommandPool(device, pci, NULL, &commandPool)) abort();

  VkCommandBuffer* commandBuffers = malloc(n_image * sizeof(*commandBuffers));
  if (!commandBuffers) abort();

  VkCommandBufferAllocateInfo cbai[1];
  memset(cbai, 0, sizeof(cbai));
  cbai->sType       = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
  cbai->commandPool = commandPool;
  cbai->level       = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
  cbai->commandBufferCount = n_image;
  if (VK_SUCCESS != vkAllocateCommandBuffers(device, cbai, commandBuffers)) abort();

  for (size_t i = 0; i < n_image; ++i) {
    VkCommandBufferBeginInfo b[1];
    memset(b, 0, sizeof(b));
    b->sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    if (VK_SUCCESS != vkBeginCommandBuffer(commandBuffers[i], b)) abort();

    VkRenderPassBeginInfo rb[1];
    memset(rb, 0, sizeof(rb));
    rb->sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
    rb->renderPass = renderPass;
    rb->framebuffer = fbs[i];
    rb->renderArea.extent = *(caps->extent);

    VkClearValue clearColor;
    clearColor.color.float32[3] = 1.0f;
    rb->clearValueCount = 1;
    rb->pClearValues = &clearColor;
    vkCmdBeginRenderPass(commandBuffers[i], rb, VK_SUBPASS_CONTENTS_INLINE);
    vkCmdBindPipeline(commandBuffers[i], VK_PIPELINE_BIND_POINT_GRAPHICS, graphicsPipeline);
    vkCmdDraw(commandBuffers[i], 3, 1, 0, 0);
    vkCmdEndRenderPass(commandBuffers[i]);
    vkEndCommandBuffer(commandBuffers[i]);
    memset(&clearColor, 0, sizeof(clearColor));
  }

  while (!glfwWindowShouldClose(win)) {
    uint32_t idx;
    vkAcquireNextImageKHR(device, swapchain, UINT64_MAX, VK_NULL_HANDLE, VK_NULL_HANDLE, &idx);

    VkSubmitInfo si[1];
    memset(si, 0, sizeof(si));
    si->commandBufferCount = 1;
    si->pCommandBuffers = commandBuffers + idx;

    vkQueueSubmit(graphics_queue, 1, si, VK_NULL_HANDLE);

    VkPresentInfoKHR pi[1];

    glfwPollEvents();
    usleep(1000);
  }

  // alright so this really sucks

done:
  vkDestroySwapchainKHR(device, swapchain, NULL);
  vkDestroyDevice(device, NULL);
  vkDestroySurfaceKHR(instance, surface, NULL);
  vkDestroyInstance(instance, NULL);
  glfwDestroyWindow(win);
  glfwTerminate();
}
