#version 450
#extension GL_ARB_separate_shader_objects : enable

// what is this black magic
layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(1.0, 0.0, 0.0, 1.0);
}

// some sort of nifty gradient, is this a texture?
// FIXME ignoring this for now because I have no idea what is going on
// vec3 colors[3] = vec3[](
//     vec3(1.0, 0.0, 0.0),
//     vec3(0.0, 1.0, 0.0),
//     vec3(0.0, 0.0, 1.0),
// );
//
// have to change the vertex shader as well to send some value about which
// colors it wants to use for this fragment, or something
