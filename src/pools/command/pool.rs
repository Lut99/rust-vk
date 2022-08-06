//  POOL.rs
//    by Lut99
// 
//  Created:
//    05 May 2022, 10:45:56
//  Last edited:
//    06 Aug 2022, 11:11:00
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains the pool implemenation for this type of pool.
// 

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

use ash::vk;

pub use crate::pools::errors::CommandPoolError as Error;
use crate::log_destroy;
use crate::auxillary::enums::CommandBufferLevel;
use crate::auxillary::flags::CommandBufferFlags;
use crate::auxillary::structs::QueueFamilyInfo;
use crate::device::Device;


/***** POPULATES *****/
/// Populates a VkCommandPoolCreateInfo struct.
/// 
/// # Arguments
/// - index: The index of the queue family where this pool will live.
/// - flags: The VkCommandPoolCreateFlags struct to create the pool with.
#[inline]
fn populate_pool_info(index: u32, flags: vk::CommandPoolCreateFlags) -> vk::CommandPoolCreateInfo {
    vk::CommandPoolCreateInfo {
        // Set the default stuff
        s_type : vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next : ptr::null(),
        flags,

        // Set the queue family index for which we allocate buffers in this pool.
        queue_family_index : index,
    }
}

/// Populates a given VkCommandBufferAllocateInfo struct.
/// 
/// # Arguments
/// - `pool`: The VkCommandPool where to allocate this new buffer.
/// - `count`: The number of buffers to allocate.
/// - `level`: The VkCommandBufferLevel struct indicating the call source.
#[inline]
fn populate_buffer_info(pool: vk::CommandPool, count: u32, level: vk::CommandBufferLevel) -> vk::CommandBufferAllocateInfo {
    vk::CommandBufferAllocateInfo {
        // Set the standard stuff
        s_type : vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next : ptr::null(),

        // Set the pool where the buffers will be allocated
        command_pool : pool,

        // Set the number of buffers to allocate
        command_buffer_count : count,

        // Finally, set the level
        level,
    }
}





/***** LIBRARY *****/
/// The CommandPool defines a Pool for command buffers.
pub struct CommandPool {
    /// The Device where the CommandPool lives.
    device : Rc<Device>,
    /// The VkCommandPools around which we wrap. There is one per queue family.
    pools  : HashMap<u32, HashMap<CommandBufferFlags, vk::CommandPool>>,
}

impl CommandPool {
    /// Constructor for the CommandPool.
    /// 
    /// # Arguments
    /// - `device`: The Device where the CommandPool will live.
    /// 
    /// # Returns
    /// A new CommandPool on success.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend could not allocate the pool for some reason.
    pub fn new(device: Rc<Device>) -> Result<Rc<RefCell<Self>>, Error> {
        // Get the family info
        let family_info: &QueueFamilyInfo = device.families();

        // Prepare one hashmap per unique queue index type
        let mut pools: HashMap<u32, HashMap<CommandBufferFlags, vk::CommandPool>> = HashMap::with_capacity(family_info.unique_len());
        for index in family_info.unique() {
            pools.insert(index, HashMap::with_capacity(1));
        }        

        // Done, wrap that and the device in the struct
        Ok(Rc::new(RefCell::new(Self {
            device,
            pools,
        })))
    }



    /// Allocate a new buffer in the pool for the given queue.
    /// 
    /// Will allocate the buffer on the appropriate underlying pool.
    /// 
    /// # Arguments
    /// - `index`: The queue family index for which we want to allocate this buffer.
    /// - `flags`: The CommandBufferFlags that allow or disallow some behaviour for Buffers. Note that, if we've never seen this combination of flags before, a new pool will be allocated to support the required types.
    /// - `level`: The CommandBufferLevel that indicates from where this CommandBuffer may be called. Has no influence on pools.
    /// 
    /// # Returns
    /// A new vk::CommandBuffer on success, with its matching vk::CommandPool (for deallocation).
    /// 
    /// # Errors
    /// If the underlying pool has no more space, then this function throws an error.
    /// 
    /// It will panic if the given queue family index is not in the user queue families when this pool was created.
    pub fn allocate(&mut self, index: u32, flags: CommandBufferFlags, level: CommandBufferLevel) -> Result<(vk::CommandPool, vk::CommandBuffer), Error> {
        // Insert a pool with these specs if it does not yet exist
        let pools: &mut HashMap<CommandBufferFlags, vk::CommandPool> = self.pools.get_mut(&index).unwrap_or_else(|| panic!("Unknown queue family index '{}'", index));
        let pool = match pools.get(&flags) {
            Some(pool) => *pool,
            None       => {
                // Populate the create info
                let pool_info = populate_pool_info(index, flags.into());

                // Allocate the pool
                let pool = unsafe {
                    match self.device.create_command_pool(&pool_info, None) {
                        Ok(pool) => pool,
                        Err(err) => { return Err(Error::CommandPoolCreateError{ err }); }
                    }  
                };

                // Store it in the pools
                pools.insert(flags, pool);

                // Done, return
                pool
            },
        };

        // Prepare the allocate info
        let buffer_info = populate_buffer_info(pool, 1, level.into());

        // ALlocate a new buffer in this pool
        let buffer = unsafe {
            match self.device.allocate_command_buffers(&buffer_info) {
                Ok(buffers) => buffers[0],
                Err(err)    => { return Err(Error::CommandBufferAllocateError{ n: 1, err }); }
            }
        };

        // Wrap it in the CommandBuffer struct and return.
        Ok((pool, buffer))
    }

    /// Allocates N new command buffers in the pool with the same properties.
    /// 
    /// # Arguments
    /// - `count`: The number of buffers to allocate.
    /// - `index`: The queue family index for which we want to allocate these buffers.
    /// - `flags`: The CommandBufferFlags that allow or disallow some behaviour for Buffers. Note that, if we've never seen this combination of flags before, a new pool will be allocated to support the required types.
    /// - `level`: The CommandBufferLevel that indicates from where these CommandBuffers may be called. Has no influence on pools.
    /// 
    /// # Returns
    /// A vector of size `count` with the new vk::CommandBuffers (and the matching vk::CommandPools (for deallocation)) on success.
    /// 
    /// # Errors
    /// If the underlying pool has no more space, then this function throws an error.
    /// 
    /// It will panic if the given queue family index is not in the user queue families when this pool was created.
    pub fn n_allocate(&mut self, count: u32, index: u32, flags: CommandBufferFlags, level: CommandBufferLevel) -> Result<Vec<(vk::CommandPool, vk::CommandBuffer)>, Error> {
        // Insert a pool with these specs if it does not yet exist
        let pools: &mut HashMap<CommandBufferFlags, vk::CommandPool> = self.pools.get_mut(&index).unwrap_or_else(|| panic!("Unknown queue family index '{}'", index));
        let pool = match pools.get(&flags) {
            Some(pool) => *pool,
            None       => {
                // Populate the create info
                let pool_info = populate_pool_info(index, flags.into());

                // Allocate the pool
                let pool = unsafe {
                    match self.device.create_command_pool(&pool_info, None) {
                        Ok(pool) => pool,
                        Err(err) => { return Err(Error::CommandPoolCreateError{ err }); }
                    }  
                };

                // Store it in the pools
                pools.insert(flags, pool);

                // Done, return
                pool
            },
        };

        // Prepare the allocate info
        let buffer_info = populate_buffer_info(pool, count, level.into());

        // ALlocate the new buffers in this pool
        unsafe {
            match self.device.allocate_command_buffers(&buffer_info) {
                Ok(buffers) => Ok(buffers.into_iter().map(|b| (pool, b)).collect()),
                Err(err)    => Err(Error::CommandBufferAllocateError{ n: 1, err }),
            }
        }
    }



    /// Trims the CommandPool.
    /// 
    /// This effectively releases owned but unused memory from the pool.
    /// 
    /// # Errors
    /// This function does not error.
    pub fn trim(&self) {
        // Call trim on all nested pools
        for pools in self.pools.values() {
            for pool in pools.values() {
                unsafe { self.device.trim_command_pool(*pool, vk::CommandPoolTrimFlags::empty()); }
            }
        }
    }

    /// Resets the CommandPool.
    /// 
    /// Doing this means that _all_ of the allocated buffers will become invalid.
    /// 
    /// # Arguments
    /// - `free_resources`: If true, then the associated memory of the pool itself will be released as well.
    /// 
    /// # Returns
    /// The same instance, but now reset.
    /// 
    /// # Errors
    /// Errors if the underlying Vulkan backend does.
    pub fn reset(self, free_resources: bool) -> Result<Self, Error> {
        // Call reset for every nested pool
        for pools in self.pools.values() {
            for pool in pools.values() {
                if let Err(err) = unsafe { self.device.reset_command_pool(*pool, if free_resources { vk::CommandPoolResetFlags::RELEASE_RESOURCES } else { vk::CommandPoolResetFlags::empty() }) } {
                    return Err(Error::CommandPoolResetError{ err });
                }
            }
        }

        // Done, return
        Ok(self)
    }



    /// Returns the parent device.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        // Destroy all the pools
        log_destroy!(self, CommandPool);
        for pools in self.pools.values() {
            for pool in pools.values() {
                unsafe { self.device.destroy_command_pool(*pool, None); }
            }
        }
    }
}
