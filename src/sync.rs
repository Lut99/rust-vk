/* SYNC.rs
 *   by Lut99
 *
 * Created:
 *   01 May 2022, 17:26:00
 * Last edited:
 *   14 May 2022, 12:42:33
 * Auto updated?
 *   Yes
 *
 * Description:
 *   Contains synchronization primitive wrappers.
**/

use std::ptr;
use std::rc::Rc;

use ash::vk;

pub use crate::errors::SyncError as Error;
use crate::log_destroy;
use crate::device::Device;


/***** POPULATE FUNCTIONS *****/
/// Creates a new VkSemaphoreCreateInfo struct.
#[inline]
fn populate_semaphore_info() -> vk::SemaphoreCreateInfo {
    vk::SemaphoreCreateInfo {
        // Only set the default stuff
        s_type : vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::SemaphoreCreateFlags::empty(),
    }
}

/// Creates a new VkFenceCreateInfo struct.
/// 
/// # Arguments
/// - `flags`: The VkFenceCreateFlags to initialize this Fence with.
#[inline]
fn populate_fence_info(flags: vk::FenceCreateFlags) -> vk::FenceCreateInfo {
    vk::FenceCreateInfo {
        // Only set the default stuff
        s_type : vk::StructureType::FENCE_CREATE_INFO,
        p_next : ptr::null(),
        flags,
    }
}





/***** LIBRARY *****/
/// Implements a Semaphore, i.e., something that gets signalled when something else is ready.
pub struct Semaphore {
    /// The device where the Semaphore lives
    device    : Rc<Device>,
    /// The Semaphore itself
    semaphore : vk::Semaphore,
}

impl Semaphore {
    /// Constructor for the Semaphore.
    /// 
    /// # Arguments
    /// - `device`: The Device where the semaphore will live.
    /// 
    /// # Returns
    /// A new Semaphore instance on success.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend could not create the Semaphore.
    pub fn new(device: Rc<Device>) -> Result<Rc<Self>, Error> {
        // Create the create info
        let semaphore_info = populate_semaphore_info();

        // Create the semaphore on the device
        let semaphore = unsafe {
            match device.create_semaphore(&semaphore_info, None) {
                Ok(semaphore) => semaphore,
                Err(err)      => { return Err(Error::SemaphoreCreateError{ err }); }
            }
        };

        // Done, wrap in an instance and return
        Ok(Rc::new(Self {
            device,
            semaphore,
        }))
    }



    /// Returns the device where this Semaphore lives.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the internal VkSemaphore.
    #[inline]
    pub fn vk(&self) -> vk::Semaphore { self.semaphore }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        log_destroy!(self, Semaphore);
        unsafe { self.device.destroy_semaphore(self.semaphore, None); }
    }
}



/// Implements a Fence, i.e., something that the CPU manually has to set to continue.
pub struct Fence {
    /// The device where the Fence lives
    device : Rc<Device>,
    /// The Fence itself
    fence  : vk::Fence,
}

impl Fence {
    /// Constructor for the Fence.
    /// 
    /// # Arguments
    /// - `device`: The Device where the semaphore will live.
    /// - `signalled`: Whether the Fence should already begin in a signalled state or not.
    /// 
    /// # Returns
    /// A new Fence instance on success.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend could not create the Fence.
    pub fn new(device: Rc<Device>, signalled: bool) -> Result<Rc<Self>, Error> {
        // Create the create info with the proper signalled state
        let fence_info = populate_fence_info(if signalled { vk::FenceCreateFlags::SIGNALED } else { vk::FenceCreateFlags::empty() });

        // Create the fence on the device
        let fence = unsafe {
            match device.create_fence(&fence_info, None) {
                Ok(fence) => fence,
                Err(err)  => { return Err(Error::FenceCreateError{ err }); }
            }
        };

        // Done, wrap in an instance and return
        Ok(Rc::new(Self {
            device,
            fence,
        }))
    }



    /// Blocks the current (CPU) thread until the Fence is signalled.
    /// 
    /// # Arguments
    /// - `timeout`: An optional timeout to wait for this Fence. A timeout of 0 is equal to polling, and a timeout of `u64::MAX` is equal to an indefinite poll.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend does or if a timeout has been reached.
    pub fn wait(&self, timeout: Option<u64>) -> Result<(), Error> {
        // Unpack the timeout
        let timeout = timeout.unwrap_or(u64::MAX);

        // Use the device function to wait
        unsafe {
            match self.device.wait_for_fences(&[self.fence], true, timeout) {
                Ok(_)                         => Ok(()),
                Err(ash::vk::Result::TIMEOUT) => Err(Error::FenceTimeout{ timeout }),
                Err(err)                      => Err(Error::FenceWaitError{ err }),
            }
        }
    }

    /// Polls the Fence if it's ready or not.
    /// 
    /// # Returns
    /// Whether or not the Fence is signalled (true) or not (false).
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend does.
    #[inline]
    pub fn poll(&self) -> Result<bool, Error> {
        // Use the device function to poll (timeout of 0)
        unsafe {
            match self.device.wait_for_fences(&[self.fence], true, 0) {
                Ok(_)                         => Ok(true),
                Err(ash::vk::Result::TIMEOUT) => Ok(false),
                Err(err)                      => Err(Error::FenceWaitError{ err }),
            }
        }
    }

    /// Resets the Fence from a signalled state to a non-signalled state.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend could not reset the Fence.
    #[inline]
    pub fn reset(&self) -> Result<(), Error> {
        unsafe {
            match self.device.reset_fences(&[self.fence]) {
                Ok(_)    => Ok(()),
                Err(err) => Err(Error::FenceResetError{ err }),
            }
        }
    }



    /// Returns the device where this Semaphore lives.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the internal VkFence.
    #[inline]
    pub fn vk(&self) -> vk::Fence { self.fence }
}

impl Drop for Fence {
    fn drop(&mut self) {
        log_destroy!(self, Fence);
        unsafe { self.device.destroy_fence(self.fence, None); }
    }
}
