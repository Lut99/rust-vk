//  BUFFERS.rs
//    by Lut99
// 
//  Created:
//    25 Jun 2022, 16:17:19
//  Last edited:
//    06 Aug 2022, 11:21:14
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines buffers that are used in the MemoryPool.
// 

use std::cell::{RefCell, RefMut};
use std::ptr;
use std::rc::Rc;

use ash::vk;

pub use crate::pools::errors::MemoryPoolError as Error;
use crate::{log_destroy, vec_as_ptr};
use crate::auxillary::enums::SharingMode;
use crate::auxillary::flags::{BufferUsageFlags, MemoryPropertyFlags};
use crate::auxillary::structs::MemoryRequirements;
use crate::device::Device;
use crate::pools::memory::spec::{Buffer, GpuPtr, HostBuffer, LocalBuffer, MemoryPool, TransferBuffer};


/***** POPULATE FUNCTIONS *****/
/// Populates the create info for a new Buffer (VkBufferCreateInfo).
/// 
/// # Arguments
/// - `usage_flags`: The VkBufferUsageFlags that determine how to use this buffer.
/// - `sharing_mode`: The VkSharingMode value that determines who can access this buffer.
/// - `queue_families`: If `sharing_mode` is `VkSharingMode::CONCURRENT`, then this list specifies the queue families who may access the buffer.
/// - `size`: The requested size (in bytes) of the Buffer. This may not be the actual size.
#[inline]
fn populate_buffer_info(usage_flags: vk::BufferUsageFlags, sharing_mode: vk::SharingMode, queue_families: &[u32], size: vk::DeviceSize) -> vk::BufferCreateInfo {
    vk::BufferCreateInfo {
        // Set the standard stuff
        s_type : vk::StructureType::BUFFER_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::BufferCreateFlags::empty(),

        // Set the usage flags
        usage : usage_flags,

        // Set the sharing mode (and eventual queue families)
        sharing_mode,
        queue_family_index_count : queue_families.len() as u32,
        p_queue_family_indices   : vec_as_ptr!(queue_families),

        // Finally, set the size
        size,
    }
}





/***** HELPER FUNCTIONS *****/
/// Creates & allocates a new vk::Buffer object.
fn create_buffer(device: &Rc<Device>, pool: &Rc<RefCell<dyn MemoryPool>>, usage_flags: BufferUsageFlags, sharing_mode: &SharingMode, mem_props: MemoryPropertyFlags, capacity: usize) -> Result<(vk::Buffer, vk::DeviceMemory, GpuPtr, MemoryRequirements), Error> {
    // Split the sharing mode
    let (vk_sharing_mode, vk_queue_family_indices) = sharing_mode.clone().into();

    // First, create a new Buffer object from the usage flags
    let buffer_info = populate_buffer_info(
        usage_flags.into(),
        vk_sharing_mode, &vk_queue_family_indices.unwrap_or(Vec::new()),
        capacity as vk::DeviceSize,
    );

    // Create the Buffer
    let buffer: vk::Buffer = unsafe {
        match device.create_buffer(&buffer_info, None) {
            Ok(buffer) => buffer,
            Err(err)   => { return Err(Error::BufferCreateError{ err }); }
        }
    };

    // Get the buffer memory type requirements
    let requirements: MemoryRequirements = unsafe { device.get_buffer_memory_requirements(buffer) }.into();

    // Allocate the memory in the pool
    let (memory, pointer): (vk::DeviceMemory, GpuPtr) = {
        // Get a lock on the pool first
        let mut lock: RefMut<dyn MemoryPool> = pool.borrow_mut();

        // Reserve the area
        lock.allocate(&requirements, mem_props)?
    };

    // Bind the memory
    unsafe {
        if let Err(err) = device.bind_buffer_memory(buffer, memory, pointer.into()) {
            return Err(Error::BufferBindError{ err });
        }
    };

    // Done! Return the relevant bits as a typle
    Ok((buffer, memory, pointer, requirements))
}





/***** LIBRARY *****/
/// The StagingBuffer is used to transfer memory to other Buffers.
pub struct StagingBuffer {
    /// The Device where the Buffer lives.
    device : Rc<Device>,
    /// The MemoryPool where the Buffer lives.
    pool   : Rc<RefCell<dyn MemoryPool>>,

    /// The VkBuffer object we wrap.
    buffer  : vk::Buffer,
    /// The bound memory area for this buffer.
    memory  : vk::DeviceMemory,
    /// The offset in that memory area for this buffer.
    ptr     : GpuPtr,

    /// The size (in bytes) of this Buffer.
    capacity     : usize,
    /// The sharing mode that determines which queue families have access to this Buffer.
    sharing_mode : SharingMode,
    /// The memory requirements of this Buffer.
    mem_req      : MemoryRequirements,
}

impl StagingBuffer {
    /// The usage flags for the StagingBuffer
    const USAGE_FLAGS: BufferUsageFlags  = BufferUsageFlags::TRANSFER_SRC;
    /// The memory property flags for the StagingBuffer
    const MEM_PROPS: MemoryPropertyFlags = MemoryPropertyFlags::HOST_VISIBLE;



    /// Constructor for the StagingBuffer.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `capacity`: The size of the buffer, in bytes. The actually allocated size may be larger due to alignment etc.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    pub fn new(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, capacity: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Create a buffer in the helper function
        let (buffer, memory, ptr, mem_req): (vk::Buffer, vk::DeviceMemory, GpuPtr, MemoryRequirements) = create_buffer(
            &device, &pool,
            Self::USAGE_FLAGS,
            &sharing_mode,
            Self::MEM_PROPS,
            capacity,
        )?;

        // Wrap it in ourselves as well as all other properties; done
        Ok(Rc::new(Self {
            device,
            pool,

            buffer,
            memory,
            ptr,

            capacity,
            sharing_mode,
            mem_req,
        }))
    }
}

impl Buffer for StagingBuffer {
    /// Returns the Device where the Buffer lives.
    #[inline]
    fn device(&self) -> &Rc<Device> { &self.device }
    
    /// Returns the MemoryPool where the Buffer's memory is allocated.
    #[inline]
    fn pool(&self) -> &Rc<RefCell<dyn MemoryPool>> { &self.pool }



    /// Returns the Vulkan vk::Buffer which we wrap.
    #[inline]
    fn vk(&self) -> vk::Buffer { self.buffer }

    /// Returns the Vulkan vk::DeviceMemory which we also wrap.
    #[inline]
    fn vk_mem(&self) -> vk::DeviceMemory { self.memory }

    /// Returns the offset of this Buffer in the DeviceMemory.
    #[inline]
    fn vk_offset(&self) -> vk::DeviceSize { self.ptr.into() }



    /// Returns the usage flags for this Buffer.
    #[inline]
    fn usage(&self) -> BufferUsageFlags { Self::USAGE_FLAGS }

    /// Returns the usage flags for this Buffer.
    #[inline]
    fn sharing_mode(&self) -> &SharingMode { &self.sharing_mode }

    /// Returns the memory requirements for this Buffer.
    #[inline]
    fn requirements(&self) -> &MemoryRequirements { &self.mem_req }

    /// Returns the memory properties of the memory underlying this Buffer.
    #[inline]
    fn properties(&self) -> MemoryPropertyFlags { Self::MEM_PROPS }

    /// Returns the actually allocated size of the buffer.
    #[inline]
    fn capacity(&self) -> usize { self.capacity }
}

impl HostBuffer for StagingBuffer {}

impl TransferBuffer for StagingBuffer {}

impl Drop for StagingBuffer {
    #[inline]
    fn drop(&mut self) {
        log_destroy!(self, StagingBuffer);

        // Destroy the buffer
        unsafe { self.device.destroy_buffer(self.buffer, None); }
        // Lock the pool to free the memory
        self.pool.borrow_mut().free(self.ptr);
    }
}



/// The VertexBuffer is used to transfer vertices to the GPU.
pub struct VertexBuffer {
    /// The Device where the Buffer lives.
    device : Rc<Device>,
    /// The MemoryPool where the Buffer lives.
    pool   : Rc<RefCell<dyn MemoryPool>>,

    /// The VkBuffer object we wrap.
    buffer  : vk::Buffer,
    /// The bound memory area for this buffer.
    memory  : vk::DeviceMemory,
    /// The offset in that memory area for this buffer.
    ptr     : GpuPtr,

    /// The size (in bytes) of this Buffer.
    capacity     : usize,
    /// The sharing mode that determines which queue families have access to this Buffer.
    sharing_mode : SharingMode,
    /// The memory requirements of this Buffer.
    mem_req      : MemoryRequirements,
}

impl VertexBuffer {
    /// The usage flags for the VertexBuffer
    const USAGE_FLAGS: BufferUsageFlags  = BufferUsageFlags::union(BufferUsageFlags::VERTEX_BUFFER, BufferUsageFlags::TRANSFER_DST);
    /// The memory property flags for the VertexBuffer
    const MEM_PROPS: MemoryPropertyFlags = MemoryPropertyFlags::DEVICE_LOCAL;



    /// Constructor for the VertexBuffer.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `capacity`: The size of the buffer, in bytes. The actually allocated size may be larger due to alignment etc.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    pub fn new(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, capacity: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Create a buffer in the helper function
        let (buffer, memory, ptr, mem_req): (vk::Buffer, vk::DeviceMemory, GpuPtr, MemoryRequirements) = create_buffer(
            &device, &pool,
            Self::USAGE_FLAGS,
            &sharing_mode,
            Self::MEM_PROPS,
            capacity,
        )?;

        // Wrap it in ourselves as well as all other properties; done
        Ok(Rc::new(Self {
            device,
            pool,

            buffer,
            memory,
            ptr,

            capacity,
            sharing_mode,
            mem_req,
        }))
    }
}

impl Buffer for VertexBuffer {
    /// Returns the Device where the Buffer lives.
    #[inline]
    fn device(&self) -> &Rc<Device> { &self.device }
    
    /// Returns the MemoryPool where the Buffer's memory is allocated.
    #[inline]
    fn pool(&self) -> &Rc<RefCell<dyn MemoryPool>> { &self.pool }



    /// Returns the Vulkan vk::Buffer which we wrap.
    #[inline]
    fn vk(&self) -> vk::Buffer { self.buffer }

    /// Returns the Vulkan vk::DeviceMemory which we also wrap.
    #[inline]
    fn vk_mem(&self) -> vk::DeviceMemory { self.memory }

    /// Returns the offset of this Buffer in the DeviceMemory.
    #[inline]
    fn vk_offset(&self) -> vk::DeviceSize { self.ptr.into() }



    /// Returns the usage flags for this Buffer.
    #[inline]
    fn usage(&self) -> BufferUsageFlags { Self::USAGE_FLAGS }

    /// Returns the usage flags for this Buffer.
    #[inline]
    fn sharing_mode(&self) -> &SharingMode { &self.sharing_mode }

    /// Returns the memory requirements for this Buffer.
    #[inline]
    fn requirements(&self) -> &MemoryRequirements { &self.mem_req }

    /// Returns the memory properties of the memory underlying this Buffer.
    #[inline]
    fn properties(&self) -> MemoryPropertyFlags { Self::MEM_PROPS }

    /// Returns the actually allocated size of the buffer.
    #[inline]
    fn capacity(&self) -> usize { self.capacity }
}

impl LocalBuffer for VertexBuffer {}

impl TransferBuffer for VertexBuffer {}

impl Drop for VertexBuffer {
    #[inline]
    fn drop(&mut self) {
        log_destroy!(self, VertexBuffer);

        // Destroy the buffer
        unsafe { self.device.destroy_buffer(self.buffer, None); }
        // Lock the pool to free the memory
        self.pool.borrow_mut().free(self.ptr);
    }
}
