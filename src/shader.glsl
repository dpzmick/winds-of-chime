#version 450
// FIXME figure out how to do this
// #extension GL_EXT_shader_8bit_storage : require // for uint8_t

// global workgroup==local workgroup for now
// no reason to define a local group, since we are just copying the
// entire thing
// layout( local_size_x = 64, local_size_y = 1, local_size_z = 1 ) in;

layout( binding = 0 ) readonly buffer in_buf_block {
  // uint8_t data[];
  uint data[];
} in_buf;

layout( binding = 1 ) buffer out_buf_block {
  // uint8_t data[];
  uint data[];
} out_buf;

// the GPU hierarchy is more or less a grid of grids of compute cores
// each compute core has a local neighborhood of cores that share fast memory
// the global memory for the GPU is further
// the local memory doesn't automatically cache the global memory?

void main() {
  // copy the portion of the input array that this task is responsible
  // for
  uvec3 me = gl_WorkGroupID;   // we are only using x
  // out_buf.data[ me[0] ] = in_buf.data[ me[0] ];

  // wait for the signal
  for( int i = 0; i < 100; ++i ) {
    if( in_buf.data[0] == 1 ) {
      // send the write
      out_buf.data[0] = 2;
    }
  }

  // send the failure
  out_buf.data[0] = 3;
}
