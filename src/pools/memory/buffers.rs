//  BUFFERS.rs
//    by Lut99
// 
//  Created:
//    25 Jun 2022, 16:17:19
//  Last edited:
//    13 Aug 2022, 12:45:02
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
use crate::auxillary::enums::{IndexType, SharingMode};
use crate::auxillary::flags::{BufferUsageFlags, MemoryPropertyFlags};
use crate::auxillary::structs::MemoryRequirements;
use crate::device::Device;

use super::spec::{Buffer, GpuPtr, HostBuffer, LocalBuffer, MemoryPool, TransferBuffer, Vertex};


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
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, capacity: usize) -> Result<Rc<Self>, Error> {
        Self::new_with_sharing_mode(device, pool, capacity, SharingMode::Exclusive)
    }

    /// Constructor for the StagingBuffer that takes a custom sharing mode.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `capacity`: The size of the buffer, in bytes. The actually allocated size may be larger due to alignment etc.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    pub fn new_with_sharing_mode(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, capacity: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
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

    /// Constructor for the StagingBuffer that initializes it based on the given Buffer.
    /// 
    /// Its device, pool and capacity are all the same as for the given buffer.
    /// 
    /// # Arguments
    /// - `buffer`: The Buffer for which to initialize this StagingBuffer.
    /// 
    /// # Returns
    /// A new StagingBuffer instance that is already wrapped in an Rc-pointer.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_for(buffer: &Rc<dyn Buffer>) -> Result<Rc<Self>, Error> {
        // Call the normal constructor with the siphoned values.
        Self::new(buffer.device().clone(), buffer.pool().clone(), buffer.capacity())
    }

    /// Constructor for the StagingBuffer that initializes it based on the given Buffer and a custom sharing mode.
    /// 
    /// Its device, pool and capacity are all the same as for the given buffer.
    /// 
    /// # Arguments
    /// - `buffer`: The Buffer for which to initialize this StagingBuffer.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Returns
    /// A new StagingBuffer instance that is already wrapped in an Rc-pointer.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_for_with_sharing_mode(buffer: &dyn Buffer, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Call the normal constructor with the siphoned values.
        Self::new_with_sharing_mode(buffer.device().clone(), buffer.pool().clone(), buffer.capacity(), sharing_mode)
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
    /// # Generic types
    /// - `V`: The Vertex that this VertexBuffer will contain. It will be used to determine the buffer's size.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_vertices`: The number of vertices that will be stored in this buffer. Based on the given Vertex, this will give the total capacity. Note that the actual capacity may be slightly higher due to alignment and such.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new<V: Vertex>(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_vertices: usize) -> Result<Rc<Self>, Error> {
        Self::new_with_sharing_mode::<V>(device, pool, n_vertices, SharingMode::Exclusive)
    }

    /// Constructor for the VertexBuffer that also accepts a custom sharing mode.
    /// 
    /// # Generic types
    /// - `V`: The Vertex that this VertexBuffer will contain. It will be used to determine the buffer's size.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_vertices`: The number of vertices that will be stored in this buffer. Based on the given Vertex, this will give the total capacity. Note that the actual capacity may be slightly higher due to alignment and such.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    pub fn new_with_sharing_mode<V: Vertex>(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_vertices: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Compute the total capacity
        let capacity: usize = n_vertices * V::vk_size();

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



/// The IndexBuffer is used to transfer vertex indices to the GPU.
pub struct IndexBuffer {
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
    /// The index type of this Buffer.
    index_type   : IndexType,
}

impl IndexBuffer {
    /// The usage flags for the IndexBuffer
    const USAGE_FLAGS: BufferUsageFlags  = BufferUsageFlags::union(BufferUsageFlags::INDEX_BUFFER, BufferUsageFlags::TRANSFER_DST);
    /// The memory property flags for the IndexBuffer
    const MEM_PROPS: MemoryPropertyFlags = MemoryPropertyFlags::DEVICE_LOCAL;



    /// Constructor for the IndexBuffer.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. Together with the `index_type`, this is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// - `type_index`: The type of the indices which are stored in this IndexBuffer. Does not only influence its capacity, but is also necessary information to convey to Vulkan.
    /// 
    /// # Returns
    /// A new IndexBuffer, complete with allocated memory and already wrapped in an Rc-pointer.
    #[inline]
    pub fn new(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize, index_type: IndexType) -> Result<Rc<Self>, Error> {
        // Relay to `new_with_sharing_mode` with the default SharingMode
        Self::new_with_sharing_mode(device, pool, n_indices, index_type, SharingMode::Exclusive)
    }

    /// Constructor for the IndexBuffer that takes a custom sharing mode.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. Together with the `index_type`, this is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// - `type_index`: The type of the indices which are stored in this IndexBuffer. Does not only influence its capacity, but is also necessary information to convey to Vulkan.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Returns
    /// A new IndexBuffer, complete with allocated memory and already wrapped in an Rc-pointer.
    pub fn new_with_sharing_mode(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize, index_type: IndexType, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Compute the total capacity
        let capacity: usize = n_indices * index_type.vk_size();

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
            index_type,
        }))
    }

    /// Constructor for the IndexBuffer that initializes it for 8-bit indices.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. This is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_u8(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize) -> Result<Rc<Self>, Error> {
        // Relay it to the normal constructor but with the 8-bit type flag
        Self::new(device, pool, n_indices, IndexType::UInt8)
    }

    /// Constructor for the IndexBuffer that initializes it for 8-bit indices and also accepts a custom sharing mode.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. This is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_u8_with_sharing_mode<I: Sized>(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Relay it to the normal constructor but with the 8-bit type flag
        Self::new_with_sharing_mode(device, pool, n_indices, IndexType::UInt8, sharing_mode)
    }

    /// Constructor for the IndexBuffer that initializes it for 16-bit indices.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. This is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_u16(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize) -> Result<Rc<Self>, Error> {
        // Relay it to the normal constructor but with the 16-bit type flag
        Self::new(device, pool, n_indices, IndexType::UInt16)
    }

    /// Constructor for the IndexBuffer that initializes it for 16-bit indices and also accepts a custom sharing mode.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. This is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_u16_with_sharing_mode<I: Sized>(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Relay it to the normal constructor but with the 16-bit type flag
        Self::new_with_sharing_mode(device, pool, n_indices, IndexType::UInt16, sharing_mode)
    }

    /// Constructor for the IndexBuffer that initializes it for 32-bit indices.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. This is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_u32(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize) -> Result<Rc<Self>, Error> {
        // Relay it to the normal constructor but with the 32-bit type flag
        Self::new(device, pool, n_indices, IndexType::UInt32)
    }

    /// Constructor for the IndexBuffer that initializes it for 32-bit indices and also accepts a custom sharing mode.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Buffer-part of the Buffer (i.e., the non-content part) will live.
    /// - `pool`: The MemoryPool where the Buffer-part of the Buffer (i.e., the content part) will live.
    /// - `n_indices`: The number of indices that may be stored in this buffer. This is used to compute the total capacity of the buffer. Note that the actual capacity may be slightly higher due to alignment and such.
    /// - `sharing_mode`: The mode of sharing the Buffer across queues.
    /// 
    /// # Errors
    /// This function may error if the buffer creation in the Vulkan backend failed.
    #[inline]
    pub fn new_u32_with_sharing_mode<I: Sized>(device: Rc<Device>, pool: Rc<RefCell<dyn MemoryPool>>, n_indices: usize, sharing_mode: SharingMode) -> Result<Rc<Self>, Error> {
        // Relay it to the normal constructor but with the 32-bit type flag
        Self::new_with_sharing_mode(device, pool, n_indices, IndexType::UInt32, sharing_mode)
    }



    /// Returns the index type for this buffer.
    /// 
    /// # Returns
    /// An IndexType that has the type of this Buffer.
    #[inline]
    pub fn index_type(&self) -> IndexType { self.index_type }
}

impl Buffer for IndexBuffer {
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

impl LocalBuffer for IndexBuffer {}

impl TransferBuffer for IndexBuffer {}

impl Drop for IndexBuffer {
    #[inline]
    fn drop(&mut self) {
        log_destroy!(self, IndexBuffer);

        // Destroy the buffer
        unsafe { self.device.destroy_buffer(self.buffer, None); }
        // Lock the pool to free the memory
        self.pool.borrow_mut().free(self.ptr);
    }
}
