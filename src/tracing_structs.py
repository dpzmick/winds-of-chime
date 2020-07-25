import struct
class Ticktock:
  __slots__ = ['start', 'end', 'tag_sz', 'tag']
  @classmethod
  def unpack(cls, b):
    (start,) = struct.unpack('=Q', b[(0):((0) + (8))])
    (end,) = struct.unpack('=Q', b[((0) + (8)):(((0) + (8)) + (8))])
    (tag_sz,) = struct.unpack('=Q', b[(((0) + (8)) + (8)):((((0) + (8)) + (8)) + (8))])
    tag = []
    for i in range(0, tag_sz):
      (tmp,) = struct.unpack('=b', b[(((((0) + (8)) + (8)) + (8)) + ((1) * (i))):((((((0) + (8)) + (8)) + (8)) + ((1) * (i))) + (1))])
      tag.append(tmp)
    return cls(start, end, tag_sz, tag)


  def __init__(self, start, end, tag_sz, tag):
    self.start = start
    self.end = end
    self.tag_sz = tag_sz
    self.tag = tag

  def pack(self):
    pass

  def __str__(self):
   return f'Ticktock(start={self.start},end={self.end},tag_sz={self.tag_sz},tag={self.tag})'

class NextImage:
  __slots__ = ['next_image_idx']
  @classmethod
  def unpack(cls, b):
    (next_image_idx,) = struct.unpack('=I', b[(0):((0) + (4))])
    return cls(next_image_idx)


  def __init__(self, next_image_idx):
    self.next_image_idx = next_image_idx

  def pack(self):
    pass

  def __str__(self):
   return f'NextImage(next_image_idx={self.next_image_idx})'

class VkTrace:
  __slots__ = ['stage_sz', 'stage', 'dt']
  @classmethod
  def unpack(cls, b):
    (stage_sz,) = struct.unpack('=I', b[(0):((0) + (4))])
    stage = []
    for i in range(0, stage_sz):
      (tmp,) = struct.unpack('=b', b[(((0) + (4)) + ((1) * (i))):((((0) + (4)) + ((1) * (i))) + (1))])
      stage.append(tmp)
    (dt,) = struct.unpack('=I', b[(((0) + (4)) + ((1) * (stage_sz))):((((0) + (4)) + ((1) * (stage_sz))) + (4))])
    return cls(stage_sz, stage, dt)


  def __init__(self, stage_sz, stage, dt):
    self.stage_sz = stage_sz
    self.stage = stage
    self.dt = dt

  def pack(self):
    pass

  def __str__(self):
   return f'VkTrace(stage_sz={self.stage_sz},stage={self.stage},dt={self.dt})'

PUP_STRUCT_IDS = {
  0: Ticktock,
  1: NextImage,
  2: VkTrace
}