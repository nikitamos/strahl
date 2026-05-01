// alloc.rs
use ash::vk;

pub struct Allocator<'dev> {
  props: vk::PhysicalDeviceMemoryProperties,
  dev:   &'dev ash::Device,
}

impl<'dev> Allocator<'dev> {
  pub fn staging_flags() -> vk::MemoryPropertyFlags {
    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_CACHED
  }
  pub fn device_flags() -> vk::MemoryPropertyFlags { vk::MemoryPropertyFlags::DEVICE_LOCAL }

  pub fn new(phy: vk::PhysicalDevice, dev: &'dev ash::Device, instance: &ash::Instance) -> Self {
    let props = unsafe { instance.get_physical_device_memory_properties(phy) };

    log::info!("available Vulkan memory types: ");
    for i in 0..props.memory_type_count {
      log::info!(
        "type {}: heap={} flags={:?}",
        i,
        props.memory_types[i as usize].heap_index,
        props.memory_types[i as usize].property_flags
      );
    }
    for i in 0..props.memory_heap_count {
      log::info!(
        "heap {}: size={} flags={:?}",
        i,
        props.memory_heaps[i as usize].size,
        props.memory_heaps[i as usize].flags
      );
    }

    Self { props, dev }
  }

  fn find_mem_type(
    &self,
    properties: vk::MemoryPropertyFlags,
    mut allowed_mem_types: u32,
  ) -> Option<u32> {
    let mut i = 0;
    while allowed_mem_types != 0 {
      if (allowed_mem_types & 0x01) != 0
        && (properties & self.props.memory_types[i].property_flags) == properties
      {
        return Some(i as u32);
      }
      allowed_mem_types >>= 1;
      i += 1;
    }
    None
  }

  pub fn allocate<T>(
    &self,
    size: vk::DeviceSize,
    flags: vk::MemoryPropertyFlags,
    allowed_mem_types: u32,
    next: Option<&mut T>,
  ) -> Option<vk::DeviceMemory>
  where
    T: ash::vk::ExtendsMemoryAllocateInfo,
  {
    let flags_str = format!("{:?}", flags);
    log::trace!(
      "requested allocation with flags: {}, supported types: 0x{:x} :: ",
      flags_str,
      allowed_mem_types
    );

    let mem_type = self.find_mem_type(flags, allowed_mem_types)?;

    log::trace!("using mem type {}", mem_type);

    let mut alloc_info = vk::MemoryAllocateInfo::default()
      .allocation_size(size)
      .memory_type_index(mem_type);
    if let Some(next) = next {
      alloc_info = alloc_info.push_next(next);
    }

    unsafe { self.dev.allocate_memory(&alloc_info, None).ok() }
  }

  pub fn dealloc(&self, mem: vk::DeviceMemory, _next: *const std::ffi::c_void) {
    unsafe {
      self.dev.free_memory(mem, None);
    }
  }
}
