from tracing_structs import *
import struct
import collections
from matplotlib import pyplot as plt

with open('/home/dpzmick/programming/winds-of-chime/build/outfile', 'rb') as f:
    m = collections.defaultdict(list)
    while True:
        b = f.read(8+4)
        if not b: break

        (tag, sz) = struct.unpack('=iQ', b)

        b = f.read(sz)
        msg = PUP_STRUCT_IDS[tag].unpack(b)

        if type(msg) == VkTrace:
            stage = ''.join(map(chr, msg.stage[:msg.stage_sz-1]))
            if stage == "transfer": continue

            if msg.dt > 100:
                m[stage].append(1e9/msg.dt)

        if type(msg) == Ticktock:
            tag = ''.join(map(chr, msg.tag[:msg.tag_sz-1]))
            if tag == "frame":
                m["fps"].append(1e9/(msg.end-msg.start))

    for k, v in m.items():
        plt.hist(v, label=k, bins=100)

    plt.legend()
    plt.show()
