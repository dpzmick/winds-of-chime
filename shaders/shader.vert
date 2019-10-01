#version 450

// this is apparently not a normal thing to do
vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

void main() {
    // what in gods name is this
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
