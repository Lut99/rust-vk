//  ERRORS.rs
//    by Lut99
// 
//  Created:
//    05 May 2022, 10:44:39
//  Last edited:
//    06 Aug 2022, 10:54:17
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains errors that relate to the pools.
// 

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use crate::auxillary::flags::{DeviceMemoryType, DeviceMemoryTypeFlags, MemoryPropertyFlags};


/***** ERRORS *****/
/// Defines errors for MemoryPools / Buffers.
#[derive(Debug)]
pub enum MemoryPoolError {
    /// Could not find a memory type with all of the supported requirements and properties.
    UnsupportedMemoryRequirements{ name: String, types: DeviceMemoryTypeFlags, props: MemoryPropertyFlags },
    /// Failed to allocate a new memory block
    MemoryAllocateError{ name: String, size: usize, mem_type: DeviceMemoryType, err: ash::vk::Result },
    /// Could not allocate a new continious block of memory due to some kind of out-of-memory error.
    OutOfMemoryError{ req_size: usize },
    /// The given memory pointer was not one matching a block to free.
    UnknownPointer{ ptr: usize },

    /// Could not allocate a CommandBuffer for some purpose.
    CommandBufferError{ what: &'static str, err: CommandPoolError },
    /// Could not start recording the temporary command buffer
    CommandBufferRecordBeginError{ what: &'static str, err: CommandPoolError },
    /// Could not finish recording the temporary command buffer
    CommandBufferRecordEndError{ what: &'static str, err: CommandPoolError },
    /// Failed to submit the command buffer
    SubmitError{ what: &'static str, err: crate::queue::Error },
    /// Failed to drain the transfer queue
    DrainError{ err: crate::queue::Error },

    /// Failed to create a new VkBuffer object.
    BufferCreateError{ err: ash::vk::Result },
    /// Failed to bind a buffer to allocated memory.
    BufferBindError{ err: ash::vk::Result },
    /// Failed to map a buffer's memory to host memory.
    BufferMapError{ err: ash::vk::Result },
    /// Failed to flush a buffer's mapped memory area.
    BufferFlushError{ err: ash::vk::Result },
}

impl Display for MemoryPoolError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use MemoryPoolError::*;
        match self {
            UnsupportedMemoryRequirements{ name, types, props } => write!(f, "Device '{}' has no memory type that supports memory requirements '{:#b}' and memory properties {}", name, u32::from(*types), props),
            MemoryAllocateError{ name, size, mem_type, err }    => write!(f, "Device '{}' could not allocate {} bytes on memory type {}: {}", name, size, u32::from(*mem_type), err),
            OutOfMemoryError{ req_size }                        => write!(f, "Could not allocate new block of {} bytes", req_size),
            UnknownPointer{ ptr }                               => write!(f, "Pointer '{:#X}' does not point to an allocated block", ptr),

            CommandBufferError{ what, err }            => write!(f, "Could not create a {} command buffer: {}", what, err),
            CommandBufferRecordBeginError{ what, err } => write!(f, "Could not start recording a {} command buffer: {}", what, err),
            CommandBufferRecordEndError{ what, err }   => write!(f, "Could not record a {} command buffer: {}", what, err),
            SubmitError{ what, err }                   => write!(f, "Could not submit {} command buffer to queue: {}", what, err),
            DrainError{ err }                          => write!(f, "Failed to drain command queue: {}", err),

            BufferCreateError{ err } => write!(f, "Could not create Buffer: {}", err),
            BufferBindError{ err }   => write!(f, "Could not bind Buffer to memory: {}", err),
            BufferMapError{ err }    => write!(f, "Could not map Buffer memory to host memory: {}", err),
            BufferFlushError{ err }  => write!(f, "Could not flush Buffer mapped memory area: {}", err),
        }
    }
}

impl Error for MemoryPoolError {}



/// Defines errors for CommandPools / CommandBuffers.
#[derive(Debug)]
pub enum CommandPoolError {
    /// Could not create the new VkCommandPool.
    CommandPoolCreateError{ err: ash::vk::Result },

    /// Could not allocate one or more new command buffers.
    CommandBufferAllocateError{ n: u32, err: ash::vk::Result },

    /// Could not reset the command pool(s).
    CommandPoolResetError{ err: ash::vk::Result },

    /// Could not begin a command buffer.
    CommandBufferBeginError{ err: ash::vk::Result },
    /// Could not end a command buffer (because something else went wrong).
    CommandBufferRecordError{ err: ash::vk::Result },
}

impl Display for CommandPoolError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use CommandPoolError::*;
        match self {
            CommandPoolCreateError{ err } => write!(f, "Could not create CommandPool: {}", err),
            
            CommandBufferAllocateError{ n, err } => write!(f, "Could not allocate {} CommandBuffer{}: {}", n, if *n == 1 { "" } else { "s" }, err),

            CommandPoolResetError{ err } => write!(f, "Could not reset CommandPool: {}", err),

            CommandBufferBeginError{ err }  => write!(f, "Could not begin CommandBuffer: {}", err),
            CommandBufferRecordError{ err } => write!(f, "Failed to record CommandBuffer: {}", err),
        }
    }
}

impl Error for CommandPoolError {}
