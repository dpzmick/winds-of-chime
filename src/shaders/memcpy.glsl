#version 450

layout(set = 0, binding = 0) buffer DataIn {
    uint data[];
} bin;

layout(set = 0, binding = 1) buffer DataOut {
    uint data[];
} bout;

void main() {
    uvec3 coord = gl_WorkGroupID;
    bout.data[coord[0]] = bin.data[coord[0]];
}
