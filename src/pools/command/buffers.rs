//  BUFFERS.rs
//    by Lut99
// 
//  Created:
//    05 May 2022, 10:45:36
//  Last edited:
//    13 Aug 2022, 12:34:22
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains the buffer definitions for this type of Pool.
// 

use std::cell::{RefCell, RefMut};
use std::ptr;
use std::rc::Rc;

use ash::vk;

pub use crate::pools::errors::CommandPoolError as Error;
use crate::log_destroy;
use crate::auxillary::enums::{BindPoint, CommandBufferLevel};
use crate::auxillary::flags::{CommandBufferFlags, CommandBufferUsageFlags};
use crate::auxillary::structs::Rect2D;
use crate::device::Device;
use crate::pipeline::Pipeline;
use crate::render_pass::RenderPass;
use crate::framebuffer::Framebuffer;
use crate::pools::memory::{Buffer, IndexBuffer, VertexBuffer};
use crate::pools::command::Pool as CommandPool;


/***** POPULATE FUNCTIONS *****/
/// Populates the begin info for recording a new command buffer.
/// 
/// # Arguments
/// - `flags`: The CommandBufferUsage flags to set for this buffer.
/// - `inheritance_info`: The VkCommandBufferInheritenceInfo struct that describes what a secondary command buffer has to inherit from a primary one. Should be NULL if this is a primary buffer.
#[inline]
fn populate_begin_info(flags: vk::CommandBufferUsageFlags, inheritance_info: *const vk::CommandBufferInheritanceInfo) -> vk::CommandBufferBeginInfo {
    vk::CommandBufferBeginInfo {
        // Do the standard stuff
        s_type : vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next : ptr::null(),
        
        // Set the flags for the buffers
        flags,

        // Set the inheritence info
        p_inheritance_info : inheritance_info,
    }
}

/// Populates a VkRenderPassBeginInfo struct.
/// 
/// # Arguments
/// - `render_pass`: The VkRenderPass to begin.
/// - `framebuffer`: The VkFramebuffer to render to in this pass.
/// - `render_area`: A VkRect2D detailling the area of the framebuffer to render to.
/// - `clear_values`: A list of 4D colour vectors that indicate the colour to reset the framebuffer for when loading it (if set so in the render pass).
#[inline]
fn populate_render_pass_begin_info(render_pass: vk::RenderPass, framebuffer: vk::Framebuffer, render_area: vk::Rect2D, clear_values: &[vk::ClearValue]) -> vk::RenderPassBeginInfo {
    vk::RenderPassBeginInfo {
        // Set default stuff
        s_type : vk::StructureType::RENDER_PASS_BEGIN_INFO,
        p_next : ptr::null(),

        // Set the render pass and framebuffer
        render_pass,
        framebuffer,

        // Set the render area
        render_area,

        // Set the list of clear values
        clear_value_count : clear_values.len() as u32,
        p_clear_values    : clear_values.as_ptr(),
    }
}





/***** LIBRARY *****/
/// The CommandBuffer is used to record various GPU commands in.
pub struct CommandBuffer {
    /// The parent CommandPool where this buffer was allocated from.
    device  : Rc<Device>,
    /// The parent CommandPool where this buffer was allocated from.
    pool    : Rc<RefCell<CommandPool>>,

    /// The parent VkCommandPool where this buffer was allocated from.
    vk_pool : vk::CommandPool,
    /// The VkCommandBuffer around which we wrap.
    buffer  : vk::CommandBuffer,
}

impl CommandBuffer {
    /// Constructor for the CommandBuffer.
    /// 
    /// This function always allocates primary CommandBuffers. To allocate secondary CommandBuffers, check `CommandBuffer::secondary()`.
    /// 
    /// # Arguments
    /// - `device`: The Device where to allocate the CommandBuffer on.
    /// - `pool`: The CommandPool where to allocate the Buffer from.
    /// - `index`: The QueueFamilyIndex for which we want to allocate this CommandBuffer.
    /// - `flags`: Any flags that are relevant when allocating CommandBuffers.
    /// 
    /// # Returns
    /// A new instance of the CommandBuffer, already wrapped in an Rc-pointer.
    /// 
    /// # Errors
    /// This function errors if the given CommandPool could not allocate a new Buffer of this type.
    pub fn new(device: Rc<Device>, pool: Rc<RefCell<CommandPool>>, index: u32, flags: CommandBufferFlags) -> Result<Rc<Self>, Error> {
        // Allocate a new vk::CommandBuffer
        let (vk_pool, buffer): (vk::CommandPool, vk::CommandBuffer) = {
            // Get a lock on the pool
            let mut lock: RefMut<CommandPool> = pool.borrow_mut();

            // Do the allocation
            lock.allocate(index, flags, CommandBufferLevel::Primary)?
        };

        // Wrap it in a struct of ourselves and done
        Ok(Rc::new(Self {
            device,
            pool,

            vk_pool,
            buffer,
        }))
    }

    /// Constructor for the CommandBuffer, which allocates it as a secondary CommandBuffer.
    /// 
    /// To allocate primary CommandBuffers, check `CommandBuffer::new()`.
    /// 
    /// # Arguments
    /// - `device`: The Device where to allocate the CommandBuffer on.
    /// - `pool`: The CommandPool where to allocate the Buffer from.
    /// - `index`: The QueueFamilyIndex for which we want to allocate this CommandBuffer.
    /// - `flags`: Any flags that are relevant when allocating CommandBuffers.
    /// 
    /// # Returns
    /// A new instance of the CommandBuffer, already wrapped in an Rc-pointer.
    /// 
    /// # Errors
    /// This function errors if the given CommandPool could not allocate a new Buffer of this type.
    pub fn secondary(device: Rc<Device>, pool: Rc<RefCell<CommandPool>>, index: u32, flags: CommandBufferFlags) -> Result<Rc<Self>, Error> {
        // Allocate a new vk::CommandBuffer
        let (vk_pool, buffer): (vk::CommandPool, vk::CommandBuffer) = {
            // Get a lock on the pool
            let mut lock: RefMut<CommandPool> = pool.borrow_mut();

            // Do the allocation
            lock.allocate(index, flags, CommandBufferLevel::Secondary)?
        };

        // Wrap it in a struct of ourselves and done
        Ok(Rc::new(Self {
            device,
            pool,

            vk_pool,
            buffer,
        }))
    }

    /// Factory method that creates N CommandBuffers at a time.
    /// 
    /// # Arguments
    /// - `device`: The Device where to allocate the CommandBuffers on.
    /// - `pool`: The CommandPool where to allocate the buffers from.
    /// - `count`: The number of new CommandBuffers to allocate.
    /// - `index`: The QueueFamilyIndex for which we want to allocate these CommandBuffers.
    /// - `flags`: Any flags that are relevant when allocating CommandBuffers.
    /// - `level`: The CommandBufferLevel of all the allocated CommandBuffers.
    /// 
    /// # Returns
    /// A Vector with `count` new instances of the CommandBuffer, already wrapped in an Rc-pointer.
    /// 
    /// # Errors
    /// This function errors if the given CommandPool could not allocate a new Buffer of this type.
    pub fn multiple(device: Rc<Device>, pool: Rc<RefCell<CommandPool>>, count: usize, index: u32, flags: CommandBufferFlags, level: CommandBufferLevel) -> Result<Vec<Rc<Self>>, Error> {
        // Allocate N new vk::CommandBuffers
        let buffers: Vec<(vk::CommandPool, vk::CommandBuffer)> = {
            // Get a lock on the pool
            let mut lock: RefMut<CommandPool> = pool.borrow_mut();

            // Do the allocation
            lock.n_allocate(count as u32, index, flags, level)?
        };

        // Map the instances and done
        Ok(buffers.into_iter().map(|(p, b)| Rc::new(Self {
            device : device.clone(),
            pool   : pool.clone(),

            vk_pool : p,
            buffer  : b,
        })).collect())
    }



    /// Prepares the CommandBuffer for recording.
    /// 
    /// # Arguments
    /// - `flags`: The CommandBufferUsageFlags that define some optional begin states.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend could not begin the command buffer.
    pub fn begin(&self, flags: CommandBufferUsageFlags) -> Result<(), Error> {
        // Populate the begin info
        let begin_info = populate_begin_info(flags.into(), ptr::null());

        // Begin the buffer
        unsafe {
            if let Err(err) = self.device.begin_command_buffer(self.buffer, &begin_info) {
                return Err(Error::CommandBufferBeginError{ err });
            }
        }

        // Success
        Ok(())
    }

    /// Records the beginning of a RenderPass.
    /// 
    /// # Arguments
    /// - `render_pass`: The RenderPass to begin.
    /// - `framebuffer`: The Framebuffer to render to in this pass.
    /// - `render_area`: A Rect2D detailling the area of the framebuffer to render to.
    /// - `clear_values`: A list of 4D colour vectors that indicate the colour to reset the framebuffer for when loading it (if set so in the render pass).
    /// 
    /// # Errors
    /// This function does not error directly, but may pass errors on to `CommandBuffer::end()`.
    pub fn begin_render_pass(&self, render_pass: &Rc<RenderPass>, framebuffer: &Rc<Framebuffer>, render_area: Rect2D<i32, u32>, clear_values: &[[f32; 4]]) {
        // Cast the clear values
        let vk_clear_values: Vec<vk::ClearValue> = clear_values.iter().map(|value| {
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: *value,
                }
            }
        }).collect();

        // Prepare the begin info
        let begin_info = populate_render_pass_begin_info(render_pass.vk(), framebuffer.vk(), render_area.into(), &vk_clear_values);

        // Begin!
        unsafe {
            self.device.cmd_begin_render_pass(self.buffer, &begin_info, vk::SubpassContents::INLINE);
        }
    }

    /// Binds the given pipeline to this RenderPass.
    /// 
    /// # Arguments
    /// - `bind_point`: The BindPoint where to bind the pipeline.
    /// - `pipeline`: The Pipeline to bind.
    /// 
    /// # Errors
    /// This function does not error directly, but may pass errors on to `CommandBuffer::end()`.
    #[inline]
    pub fn bind_pipeline(&self, bind_point: BindPoint, pipeline: &Rc<Pipeline>) {
        unsafe {
            self.device.cmd_bind_pipeline(self.buffer, bind_point.into(), pipeline.vk());
        }
    }

    /// Binds a single vertex buffer for the next `CommandBuffer::draw()`-call.
    /// 
    /// # Arguments
    /// - `index`: The binding index for the given buffer.
    /// - `vertex_buffer`: The VertexBuffers to bind.
    #[inline]
    pub fn bind_vertex_buffer(&self, index: usize, vertex_buffer: &Rc<VertexBuffer>) {
        // Call the function
        self.bind_vertex_buffers(index, &[ vertex_buffer ])
    }

    /// Binds a given list of vertex buffers for the next `CommandBuffer::draw()`-call.
    /// 
    /// # Arguments
    /// - `index`: The first binding index for the given buffers.
    /// - `vertex_buffesr`: The list of VertexBuffers to bind.
    pub fn bind_vertex_buffers(&self, index: usize, vertex_buffers: &[&Rc<VertexBuffer>]) {
        // Extract the required properties into two arrays
        let buffers: Vec<vk::Buffer>     = vertex_buffers.iter().map(|b| b.vk()).collect();
        let offsets: Vec<vk::DeviceSize> = vertex_buffers.iter().map(|b| b.vk_offset()).collect();

        // Call the function
        unsafe {
            self.device.cmd_bind_vertex_buffers(self.buffer, index as u32, &buffers, &offsets);
        }
    }

    /// Binds a single index buffer for the next `CommandBuffer::draw()`-call.
    /// 
    /// # Arguments
    /// - `index_buffer`: The IndexBuffers to bind.
    #[inline]
    pub fn bind_index_buffer(&self, index_buffer: &Rc<IndexBuffer>) {
        // Call the function
        unsafe {
            self.device.cmd_bind_index_buffer(self.buffer, index_buffer.vk(), index_buffer.vk_offset(), index_buffer.index_type().into());
        }
    }

    /// Records a draw call.
    /// 
    /// # Arguments
    /// - `n_vertices`: The number of vertices to draw.
    /// - `n_instances`: The number of instances to draw.
    /// - `first_vertex`: The position of the first vertex in the buffer to draw.
    /// - `first_instance`: The position of the first instance in the buffer to draw.
    /// 
    /// # Errors
    /// This function does not error directly, but may pass errors on to `CommandBuffer::end()`.
    #[inline]
    pub fn draw(&self, n_vertices: u32, n_instances: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.device.cmd_draw(self.buffer, n_vertices, n_instances, first_vertex, first_instance);
        }
    }

    /// Records a draw call that also uses an index buffer.
    /// 
    /// # Arguments
    /// - `n_indices`: The number of indices to draw.
    /// - `n_instances`: The number of instances to draw.
    /// - `first_index`: The position of the first index in the buffer to draw.
    /// - `vertex_offset`: An offset to apply to all indices into the vertex buffer.
    /// - `first_instance`: The position of the first instance in the buffer to draw.
    /// 
    /// # Errors
    /// This function does not error directly, but may pass errors on to `CommandBuffer::end()`.
    #[inline]
    pub fn draw_indexed(&self, n_indices: u32, n_instances: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        unsafe {
            self.device.cmd_draw_indexed(self.buffer, n_indices, n_instances, first_index, vertex_offset, first_instance);
        }
    }

    /// Records the end of a RenderPass.
    /// 
    /// # Errors
    /// This function does not error directly, but may pass errors on to `CommandBuffer::end()`.
    #[inline]
    pub fn end_render_pass(&self) {
        unsafe {
            self.device.cmd_end_render_pass(self.buffer);
        }
    }

    /// Ends recording in the CommandBuffer.
    /// 
    /// # Errors
    /// This function errors if any of the other record steps that delayed any errors has errored.
    pub fn end(&self) -> Result<(), Error> {
        unsafe {
            if let Err(err) = self.device.end_command_buffer(self.buffer) {
                return Err(Error::CommandBufferRecordError{ err });
            }
        }
        Ok(())
    }



    /// Returns the parent Device where this buffer lives.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the parent Pool where this buffer lives.
    #[inline]
    pub fn pool(&self) -> &Rc<RefCell<CommandPool>> { &self.pool }

    /// Returns the internal buffer.
    #[inline]
    pub fn vk(&self) -> vk::CommandBuffer { self.buffer }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        // Call free on the parent pool
        log_destroy!(self, CommandBuffer);
        unsafe { self.device.free_command_buffers(self.vk_pool, &[self.buffer]); }
    }
}
