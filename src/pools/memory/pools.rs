//  POOLS.rs
//    by Lut99
// 
//  Created:
//    25 Jun 2022, 18:04:08
//  Last edited:
//    06 Aug 2022, 12:07:52
//  Auto updated?
//    Yes
// 
//  Description:
//!   The pools that we use to allocate new bits of memory.
// 

use std::cell::RefCell;
use std::rc::Rc;
use std::slice;

use ash::vk;

use crate::warn;
pub use crate::pools::errors::MemoryPoolError as Error;
use crate::auxillary::flags::{DeviceMemoryType, MemoryPropertyFlags};
use crate::auxillary::structs::MemoryRequirements;
use crate::device::Device;
use crate::pools::memory::block::MemoryBlock;
use crate::pools::memory::spec::{GpuPtr, MemoryPool};


/***** UNIT TESTS *****/
#[cfg(test)]
mod tests {
    use std::cell::RefMut;
    use semver::Version;
    use crate::auxillary::flags::DeviceMemoryTypeFlags;
    use crate::auxillary::structs::DeviceFeatures;
    use crate::instance::Instance;
    use super::*;

    /// The instance extensions to use for the tests
    const INSTANCE_EXTENSIONS: &[&'static str] = &[];
    /// The instance layers to use for the tests
    const INSTANCE_LAYERS: &[&'static str]     = &[];
    /// The device extensions to use for the tests
    const DEVICE_EXTENSIONS: &[&'static str]   = &[];
    /// The device layers to use for the tests
    const DEVICE_LAYERS: &[&'static str]       = &[];
    /// The device layers to use for the tests
    const DEVICE_FEATURES: DeviceFeatures      = DeviceFeatures::cdefault();

    /// Tests the linearpool's allocation algorithm
    #[test]
    fn test_linear_pool() {
        // Initialize an instance and a device
        let instance = Instance::new(
            format!("{}_test_linear_pool", file!()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("Could not parse CARGO version"),
            format!("{}_test_linear_pool_engine", file!()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("Could not parse CARGO version"),
            INSTANCE_EXTENSIONS,
            INSTANCE_LAYERS,
        ).expect("Failed to initialize Instance");
        let device = Device::new(
            instance.clone(),
            Device::auto_select(
                instance.clone(),
                &DEVICE_EXTENSIONS,
                &DEVICE_LAYERS,
                &DEVICE_FEATURES,
            ).expect("Could not find a suitable GPU for tests"),
            &DEVICE_EXTENSIONS,
            &DEVICE_LAYERS,
            &DEVICE_FEATURES,
        ).expect("Failed to initialize Device");

        // Create a LinearPool on said device
        let pool = LinearPool::new(device.clone(), 512);
        let mut mpool: RefMut<LinearPool> = pool.borrow_mut();
        // Allocate four non-aligned blocks of 128 bytes
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 0));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 128));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 256, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 256));

        // Create another to check it overflow correctly
        let pool = LinearPool::new(device.clone(), 512);
        let mut mpool: RefMut<LinearPool> = pool.borrow_mut();
        // Allocate a block that's always too large
        match mpool.allocate(&MemoryRequirements{ align: 1, size: 1024, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }
        // Next, allocate some blocks and then check out-of-bounds
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 129, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        match mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }

        // Finally, a block to check alignment
        let pool = LinearPool::new(device.clone(), 512);
        let mut mpool: RefMut<LinearPool> = pool.borrow_mut();
        // Allocate the first block with  weird size
        let (_, _)       = mpool.allocate(&MemoryRequirements{ align: 1, size: 133, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        // Allocate one that needs to be aligned
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 4, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 136));
        // One with even bigger alignment
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 16, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 272));
        // This one should fail _because_ of its alignment
        match mpool.allocate(&MemoryRequirements{ align: 32, size: 112, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 112, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 400));

        // If we now reset this pool, we should then be able to allocate new blocks
        mpool.reset();
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 0));
    }

    /// Tests the blockpool's allocation algorithm
    #[test]
    fn test_block_pool() {
        // Initialize an instance and a device
        let instance = Instance::new(
            format!("{}_test_block_pool", file!()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("Could not parse CARGO version"),
            format!("{}_test_block_pool_engine", file!()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("Could not parse CARGO version"),
            INSTANCE_EXTENSIONS,
            INSTANCE_LAYERS,
        ).expect("Failed to initialize Instance");
        let device = Device::new(
            instance.clone(),
            Device::auto_select(
                instance.clone(),
                &DEVICE_EXTENSIONS,
                &DEVICE_LAYERS,
                &DEVICE_FEATURES,
            ).expect("Could not find a suitable GPU for tests"),
            &DEVICE_EXTENSIONS,
            &DEVICE_LAYERS,
            &DEVICE_FEATURES,
        ).expect("Failed to initialize Device");

        // Create a BlockPool on said device
        let pool = BlockPool::new(device.clone(), MemoryBlock::allocate(device.clone(), &MemoryRequirements{ align: 1, size: 512, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Could not allocate block pool memory block"));
        let mut mpool: RefMut<BlockPool> = pool.borrow_mut();
        // Allocate four non-aligned blocks of 128 bytes
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 0));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 128));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 256, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 256));

        // Create another to check it overflow correctly
        let pool = BlockPool::new(device.clone(), MemoryBlock::allocate(device.clone(), &MemoryRequirements{ align: 1, size: 512, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Could not allocate block pool memory block"));
        let mut mpool: RefMut<BlockPool> = pool.borrow_mut();
        // Allocate a block that's always too large
        match mpool.allocate(&MemoryRequirements{ align: 1, size: 1024, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }
        // Next, allocate some blocks and then check out-of-bounds
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 129, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        match mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }

        // A block to check alignment
        let pool = BlockPool::new(device.clone(), MemoryBlock::allocate(device.clone(), &MemoryRequirements{ align: 1, size: 512, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Could not allocate block pool memory block"));
        let mut mpool: RefMut<BlockPool> = pool.borrow_mut();
        // Allocate the first block with  weird size
        let (_, _)       = mpool.allocate(&MemoryRequirements{ align: 1, size: 133, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        // Allocate one that needs to be aligned
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 4, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 136));
        // One with even bigger alignment
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 16, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 272));
        // This one should fail _because_ of its alignment
        match mpool.allocate(&MemoryRequirements{ align: 32, size: 112, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 112, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 400));

        // If we now reset this pool, we should then be able to allocate new blocks
        mpool.reset();
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 0));

        // Finally we do a pool to check if it properly frees
        let pool = BlockPool::new(device.clone(), MemoryBlock::allocate(device.clone(), &MemoryRequirements{ align: 1, size: 512, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Could not allocate block pool memory block"));
        let mut mpool: RefMut<BlockPool> = pool.borrow_mut();
        // Allocate three blocks of 128 bytes
        let (_, _       ) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        let (_, pointer2) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        let (_, pointer3) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        // Free the second
        mpool.free(pointer2);
        // Where we expect the new pointer to be allocated we don't know, but we should be able to allocate at least two
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fourth block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fifth block");
        // Free the third now
        mpool.free(pointer3);
        // This one fails bc not enough space
        match mpool.allocate(&MemoryRequirements{ align: 1, size: 129, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }
        // This _two_ succeed again
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 37, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fourth block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 4, size: 60, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fourth block");
    }

    /// Tests the metapool's allocation algorithm
    #[test]
    fn test_meta_pool() {
        // Initialize an instance and a device
        let instance = Instance::new(
            format!("{}_test_block_pool", file!()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("Could not parse CARGO version"),
            format!("{}_test_block_pool_engine", file!()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("Could not parse CARGO version"),
            INSTANCE_EXTENSIONS,
            INSTANCE_LAYERS,
        ).expect("Failed to initialize Instance");
        let device = Device::new(
            instance.clone(),
            Device::auto_select(
                instance.clone(),
                &DEVICE_EXTENSIONS,
                &DEVICE_LAYERS,
                &DEVICE_FEATURES,
            ).expect("Could not find a suitable GPU for tests"),
            &DEVICE_EXTENSIONS,
            &DEVICE_LAYERS,
            &DEVICE_FEATURES,
        ).expect("Failed to initialize Device");

        // Create a MetaPool on said device
        let pool = MetaPool::new(device.clone(), 2048);
        let mut mpool: RefMut<MetaPool> = pool.borrow_mut();
        // Allocate four non-aligned blocks of 128 bytes
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 0));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 128));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 256, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 256));

        // Create another to check it overflow correctly
        let pool = MetaPool::new(device.clone(), 2048);
        let mut mpool: RefMut<MetaPool> = pool.borrow_mut();
        // Allocate a block that's always too large
        match mpool.allocate(&MemoryRequirements{ align: 1, size: usize::MAX, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()) {
            Ok(_)                              => { panic!("Pool successfully allocated block that should throw out-of-memory"); },
            Err(Error::OutOfMemoryError{ .. }) => {},
            Err(err)                           => { panic!("Memory allocation failed: {}", err); },
        }

        // A block to check alignment
        let pool = MetaPool::new(device.clone(), 2048);
        let mut mpool: RefMut<MetaPool> = pool.borrow_mut();
        // Allocate the first block with  weird size
        let (_, _)       = mpool.allocate(&MemoryRequirements{ align: 1, size: 133, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        // Allocate one that needs to be aligned
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 4, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 136));
        // One with even bigger alignment
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 16, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 272));
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 112, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 400));

        // If we now reset this pool, we should then be able to allocate new blocks
        mpool.reset();
        let (_, pointer) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        assert_eq!(pointer, GpuPtr::new(0, 0, 0));

        // Finally we do a pool to check if it properly frees
        let pool = MetaPool::new(device.clone(), 2048);
        let mut mpool: RefMut<MetaPool> = pool.borrow_mut();
        // Allocate three blocks of 128 bytes
        let (_, _       ) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate first block");
        let (_, pointer2) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate second block");
        let (_, pointer3) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
        // Free the second
        mpool.free(pointer2);
        // Where we expect the new pointer to be allocated we don't know, but we should be able to allocate at least two
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fourth block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fifth block");
        // Free the third now
        mpool.free(pointer3);
        // This _two_ succeed again
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 37, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fourth block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 4, size: 60, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::empty()).expect("Failed to allocate fourth block");

        // It can also allocate multiple blocks of memory
        // NOTE: Might want to remove this, especially the last one
        let pool = MetaPool::new(device.clone(), 2048);
        let mut mpool: RefMut<MetaPool> = pool.borrow_mut();
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::HOST_COHERENT).expect("Failed to allocate first block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::all() }, MemoryPropertyFlags::DEVICE_LOCAL).expect("Failed to allocate second block");
        let (_, _) = mpool.allocate(&MemoryRequirements{ align: 1, size: 128, types: DeviceMemoryTypeFlags::from(2 as u32) }, MemoryPropertyFlags::empty()).expect("Failed to allocate third block");
    }
}





/***** HELPER TRAITS *****/
/// Allows an unwrap that does not require 'Debug' to be defined.
pub trait DiscreetUnwrap<T, E> {
    /// Unwraps the given value, throwing a generic panic instead of an error-specific one (forgoing the need to have the Error implement 'Debug').
    fn dunwrap(self) -> T;
}

impl<T, E> DiscreetUnwrap<T, E> for Result<T, E> {
    /// Unwraps the given value, throwing a generic panic instead of an error-specific one (forgoing the need to have the Error implement 'Debug').
    fn dunwrap(self) -> T {
        match self {
            Ok(r)  => r,
            Err(_) => { panic!("Failed to unwrap result"); }
        }
    }
}





/***** HELPER STRUCTS *****/
/// Groups the BlockPools belonging to one type.
struct MemoryType {
    /// The list of pools that are allocated for this type.
    pools : Vec<BlockPool>,
    /// The index of this type
    index : DeviceMemoryType,
    /// The supported properties by this type.
    props : MemoryPropertyFlags,
}





/***** LIBRARY *****/
/// A LinearPool uses a very fast memory allocation algorithm, but wastes space because freed blocks cannot be re-used until the pool is reset. Additionally, this type of pool only supports one type of memory.
pub struct LinearPool {
    /// The Device where the LinearPool lives.
    device : Rc<Device>,
    /// The single memory block used in the linear pool.
    block  : Option<MemoryBlock>,

    /// The pointer that determines up to where we already gave to memory blocks.
    pointer  : GpuPtr,
    /// The size (in bytes) of the LinearPool.
    capacity : usize,
}

impl LinearPool {
    /// Constructor for the LinearPool.
    /// 
    /// Note that memory will be allocated lazily.
    /// 
    /// # Arguments
    /// - `capacity`: The size (in bytes) of the pool.
    /// 
    /// # Returns
    /// A new LinearPool instance, already wrapped in an Rc and a RefCell.
    #[inline]
    pub fn new(device: Rc<Device>, capacity: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            device,
            block : None,

            pointer : GpuPtr::default(),
            capacity,
        }))
    }



    /// Frees the internal memory block.
    /// 
    /// This is useful if you want to repurpose the LinearPool for a different kind of memory.
    /// 
    /// # Results
    /// Nothing, but does free the internal block so it will be allocated again on the next allocate() call.
    #[inline]
    pub fn release(&mut self) {
        self.block = None;
    }



    /// Returns the used size in the LinearPool.
    #[inline]
    pub fn size(&self) -> usize { self.pointer.into() }

    /// Returns the total size of the LinearPool.
    #[inline]
    pub fn capacity(&self) -> usize { self.capacity }
}

impl MemoryPool for LinearPool {
    /// Returns a newly allocated area of (at least) the requested size.
    /// 
    /// # Arguments
    /// - `reqs`: The memory requirements of the new memory block.
    /// - `props`: Any desired memory properties for this memory block.
    /// 
    /// # Returns
    /// A tuple with the VkDeviceMemory where the new block of memory is allocated on `.0`, and the index in this memory block on `.1`.
    /// 
    /// # Errors
    /// This function errors if the MemoryPool failed to allocate new memory.
    fn allocate(&mut self, reqs: &MemoryRequirements, props: MemoryPropertyFlags) -> Result<(vk::DeviceMemory, GpuPtr), Error> {
        // Check whether we have a block of memory already
        let memory: vk::DeviceMemory = match self.block.as_ref() {
            Some(block) => {
                // Make sure the requirements & properties are satisfied
                if !reqs.types.check(block.mem_type().into()) { panic!("LinearPool is allocated for device memory type {}, but new allocation only supports {}", block.mem_type(), reqs.types); }
                if !block.mem_props().check(props) { panic!("LinearPool is allocated for device memory type {} which supports the properties {}, but new allocation requires {}", block.mem_type(), block.mem_props(), props); }
                block.vk()
            },

            None => {
                // Allocate a new block
                let block = MemoryBlock::allocate(self.device.clone(), &reqs, props)?;
                let memory = block.vk();
                self.block = Some(block);
                memory
            },
        };

        // Compute the alignment requirements based on the current pointer
        let pointer = self.pointer.align(reqs.align);

        // Check if that leaves us with enough space
        if reqs.size > self.capacity - usize::from(pointer) { return Err(Error::OutOfMemoryError{ req_size: reqs.size }); }

        // Advance the internal pointer and return the allocated one
        self.pointer = pointer + reqs.size;
        Ok((memory, pointer))
    }

    /// Frees an allocated bit of memory.
    /// 
    /// Note that not all types of pools may actually do anything with this. A LinearPool, for example, might deallocate but will never re-use that memory until reset anyway.
    /// 
    /// # Arguments
    /// - `pointer`: The pointer to the block that was allocated.
    /// 
    /// # Panics
    /// This function may panic if the given pointer was never allocated with this pool.
    #[inline]
    fn free(&mut self, _pointer: GpuPtr) {
        warn!("Calling `LinearPool::free()` has no effect");
    }

    /// Resets the memory pool back to its initial, empty state.
    #[inline]
    fn reset(&mut self) { self.pointer = GpuPtr::default(); }



    /// Returns the device of the pool.
    #[inline]
    fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the used space in the pool.
    #[inline]
    fn size(&self) -> usize { self.pointer.into() }

    /// Returns the total space in the pool.
    #[inline]
    fn capacity(&self) -> usize { self.capacity }
}



/// A BlockPool uses a more complicated and slow allocation algorithm, but saves space because it does reuse freed blocks. This specific type of pool only supports one type of memory.
pub struct BlockPool {
    /// The Device where the BlockPool lives.
    device : Rc<Device>,
    /// The single memory block used in this pool.
    block  : MemoryBlock,

    /// The list of free blocks in the BlockPool.
    /// 
    /// Elements are of the shape:
    /// - `.0`: The offset of the block compared to the MemoryBlock.
    /// - `.1`: The size of the block (in bytes).
    free : Vec<(GpuPtr, usize)>,
    /// The list of used blocks in the BlockPool.
    /// 
    /// Elements are of the shape:
    /// - `.0`: The offset of the block compared to the MemoryBlock.
    /// - `.1`: The size of the block (in bytes).
    used : Vec<(GpuPtr, usize)>,
    /// The used space in the BlockPool.
    size : usize,
}

impl BlockPool {
    /// Constructor for the BlockPool.
    /// 
    /// # Arguments
    /// - `block`: The already allocated MemoryBlock. If you have yet to allocate one, check `MemoryBlock::allocate()`.
    /// 
    /// # Returns
    /// A new BlockPool instance, already wrapped in an Rc and a RefCell.
    pub fn new(device: Rc<Device>, block: MemoryBlock) -> Rc<RefCell<Self>> {
        // Get the new capacity
        let capacity = block.mem_size();

        // Done
        Rc::new(RefCell::new(Self {
            device,
            block,

            free : vec![ (GpuPtr::default(), capacity) ],
            used : Vec::with_capacity(1),
            size : 0,
        }))
    }
}

impl MemoryPool for BlockPool {
    /// Returns a newly allocated area of (at least) the requested size.
    /// 
    /// # Arguments
    /// - `reqs`: The memory requirements of the new memory block.
    /// - `props`: Any desired memory properties for this memory block.
    /// 
    /// # Returns
    /// A tuple with the VkDeviceMemory where the new block of memory is allocated on `.0`, and the index in this memory block on `.1`.
    /// 
    /// # Errors
    /// This function errors if the MemoryPool failed to allocate new memory.
    fn allocate(&mut self, reqs: &MemoryRequirements, props: MemoryPropertyFlags) -> Result<(vk::DeviceMemory, GpuPtr), Error> {
        // Make sure the requirements & properties are satisfied
        if !reqs.types.check(self.block.mem_type().into()) { panic!("BlockPool is allocated for device memory type {}, but new allocation only supports {}", self.block.mem_type(), reqs.types); }
        if !self.block.mem_props().check(props) { panic!("BlockPool is allocated for device memory type {} which supports the properties {}, but new allocation requires {}", self.block.mem_type(), self.block.mem_props(), props); }

        // Optimization: we can stop early if there is no more space
        if reqs.size > self.block.mem_size() { return Err(Error::OutOfMemoryError{ req_size: reqs.size }); }

        // Now, search for a free block with enough size
        let mut new_used: (GpuPtr, usize) = (GpuPtr::null(), reqs.size);
        let mut remove: Option<usize> = None;
        for (i, (block_ptr, block_size)) in self.free.iter_mut().enumerate() {
            // Compute the aligned pointer for this block
            let align_ptr: GpuPtr = block_ptr.align(reqs.align);

            // Take that into account with the aligned size
            let new_size: usize = (align_ptr.ptr() - block_ptr.ptr()) as usize + reqs.size;
            if new_size <= *block_size {
                // Set the pointer of the new used block
                new_used.0 = align_ptr;

                // Split the block in a used block and shrink the free block.
                *block_ptr  += new_size;
                *block_size -= new_size;
                // Mark for removal if that leaves us with an empty block
                if *block_size == 0 { remove = Some(i); }

                // Quit for speedz
                break;
            }

            // If not enough size, try the next one
        }
        if new_used.0.is_null() { return Err(Error::OutOfMemoryError{ req_size: reqs.size }); }
        // If needed, remove the free block
        if let Some(index) = remove { self.free.swap_remove(index); }

        // Insert the new used block
        if self.used.len() == self.used.capacity() { self.used.reserve(self.used.capacity()); }
        self.used.push(new_used);

        // Update the size and we're done
        self.size += new_used.1;
        println!("Blocks after allocate:\n - Used: {:?}\n - Free: {:?}", self.used, self.free);
        Ok((self.block.vk(), new_used.0))
    }

    /// Frees an allocated bit of memory.
    /// 
    /// Note that not all types of pools may actually do anything with this. A LinearPool, for example, might deallocate but will never re-use that memory until reset anyway.
    /// 
    /// # Arguments
    /// - `pointer`: The pointer to the block that was allocated.
    /// 
    /// # Panics
    /// This function may panic if the given pointer was never allocated with this pool.
    #[inline]
    fn free(&mut self, pointer: GpuPtr) {
        // Search the used blocks for a matching allocation
        let mut new_free: (GpuPtr, usize) = (GpuPtr::null(), 0);
        let mut remove: usize = 0;
        for (i, (block_ptr, block_size)) in self.used.iter_mut().enumerate() {
            if *block_ptr == pointer {
                // Mark this one for removal, update the new free
                new_free = (*block_ptr, *block_size);
                remove   = i;
                break;
            }
        }
        if new_free.0.is_null() { panic!("Given pointer '{:?}' was not allocated with this pool", pointer); }
        self.used.swap_remove(remove);

        // Add the new free to the list
        if self.free.len() == self.free.capacity() { self.free.reserve(self.free.capacity()); }
        self.free.push(new_free);

        // Update the size, done
        self.size -= new_free.1;
        println!("Blocks after free:\n - Used: {:?}\n - Free: {:?}", self.used, self.free);
    }

    /// Resets the memory pool back to its initial, empty state.
    #[inline]
    fn reset(&mut self) {
        // Clear the lists
        self.used.clear();
        self.free.clear();
        self.free.push((GpuPtr::default(), self.block.mem_size()));

        // Reset the size
        self.size = 0;
    }



    /// Returns the device of the pool.
    #[inline]
    fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the used space in the pool.
    #[inline]
    fn size(&self) -> usize { self.size }

    /// Returns the total space in the pool.
    #[inline]
    fn capacity(&self) -> usize { self.block.mem_size() }
}



/// A MetaPool is a dynamic collection of BlockPools such that it allows allocating for any device memory type.
pub struct MetaPool {
    /// The device where all nested pools live.
    device: Rc<Device>,

    /// The preferred size of a new pool. Note that pools may actually be smaller or larger, but this is the default size.
    pref_size  : usize,
    /// A collection of memory types supported by this GPU.
    types      : Vec<MemoryType>,

    /// The total used size in the MetaPool.
    size     : usize,
    /// The total capacity in the MetaPool (estimation).
    capacity : usize,
}

impl MetaPool {
    /// Constructor for the MetaPool.
    /// 
    /// This constructor analyses the given device for quite some things and locks those in memory for the duration of its lifetime. If the memory properties are prone to change (somehow), consider creating the pool closer to where you need it.
    /// 
    /// # Arguments
    /// - `device`: The Device where all memory will be allocated.
    /// - `pref_size`: The preferred memory block size. Note that blocks may still be smaller (to fill gaps) or larger (for larger allocations).
    /// 
    /// # Returns
    /// A new MetaPool instance, wrapped in a reference-counting pointer.
    pub fn new(device: Rc<Device>, pref_size: usize) -> Rc<RefCell<Self>> {
        // Get all available types from the device
        let device_props: vk::PhysicalDeviceMemoryProperties = unsafe { device.instance().get_physical_device_memory_properties(device.physical_device()) };
        let device_heaps: &[vk::MemoryHeap] = unsafe { slice::from_raw_parts(device_props.memory_heaps.as_ptr(), device_props.memory_heap_count as usize) };
        let device_types: &[vk::MemoryType] = unsafe { slice::from_raw_parts(device_props.memory_types.as_ptr(), device_props.memory_type_count as usize) };

        // Deduce both the list and the total capacity
        let mut types: Vec<MemoryType> = Vec::with_capacity(device_types.len());
        let mut capacity: usize = 0;
        for (i, mem_type) in device_types.into_iter().enumerate() {
            capacity += device_heaps[mem_type.heap_index as usize].size as usize;
            types.push(MemoryType {
                pools : Vec::with_capacity(4),
                index : DeviceMemoryType::from(i as u32),
                props : mem_type.property_flags.into(),
            })
        }

        // Wrap that in ourselves and done
        Rc::new(RefCell::new(Self {
            device,

            pref_size,
            types,

            size : 0,
            capacity,
        }))
    }
}

impl MemoryPool for MetaPool {
    /// Returns a newly allocated area of (at least) the requested size.
    /// 
    /// The memory allocation algorithm used is as follows (Taken from the VMA:
    /// <https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/general_considerations.html>):
    ///  1. Try to find free range of memory in existing blocks.
    ///  2. If failed, try to create a new block of VkDeviceMemory, with preferred 
    ///     block size.
    ///  3. If failed, try to create such block with size / 2, size / 4, size / 8.
    ///  // 4. If failed, try to allocate separate VkDeviceMemory for this
    ///  //   allocation.
    ///  5. If failed, choose other memory type that meets the requirements
    ///     specified in VmaAllocationCreateInfo and go to point 1.
    ///  6. If failed, return out-of-memory error.
    /// 
    /// # Arguments
    /// - `reqs`: The memory requirements of the new memory block.
    /// - `props`: Any desired memory properties for this memory block.
    /// 
    /// # Returns
    /// A tuple with the VkDeviceMemory where the new block of memory is allocated on `.0`, and the index in this memory block on `.1`.
    /// 
    /// # Errors
    /// This function errors if the MemoryPool failed to allocate new memory.
    fn allocate(&mut self, reqs: &MemoryRequirements, props: MemoryPropertyFlags) -> Result<(vk::DeviceMemory, GpuPtr), Error> {
        // Preparation: construct a list of types that favours those we have already allocated from
        let mut memory_types: Vec<&mut MemoryType> = Vec::with_capacity(self.types.len());
        let mut unused_types: Vec<&mut MemoryType> = Vec::with_capacity(self.types.len());
        for mem_type in &mut self.types {
            if !mem_type.pools.is_empty() { memory_types.push(mem_type); }
            else                          { unused_types.push(mem_type); }
        }
        memory_types.append(&mut unused_types);

        // 1. Iterate over the blocks to find if any existing block suits us
        for mem_type in memory_types {
            // Skip if not in the allowed types or not supporting the correct properties
            if !reqs.types.check(mem_type.index.into()) { continue; }
            if !mem_type.props.check(props)      { continue; }

            // Now try to find a pool with enough space
            for (i, pool) in &mut mem_type.pools.iter_mut().enumerate() {
                // Skip if not enough space
                if reqs.size > pool.capacity() - pool.size() { continue; }

                // Attempt to allocate a new block here and encode the pool index in the pointer
                let (memory, mut pointer): (vk::DeviceMemory, GpuPtr) = pool.allocate(reqs, props)?;
                pointer.set_type_idx(u32::from(mem_type.index) as u8);
                pointer.set_pool_idx(i                         as u16);
                return Ok((memory, pointer));
            }

            // 2. & 3. If we failed to find an appropriate pool, then (try to) allocate a new one
            for block_size in [ self.pref_size, self.pref_size / 2, self.pref_size / 4, self.pref_size / 8, reqs.size ] {
                // Stop trying if the block isn't large enough for this allocation
                if reqs.size > block_size { continue; }

                // Attempt the allocation
                let new_block: MemoryBlock = match MemoryBlock::allocate_on_type(self.device.clone(), mem_type.index, block_size) {
                    Ok(new_block)                      => new_block,
                    // Try the next block if not enough memory, or fail otherwise
                    Err(Error::OutOfMemoryError{ .. }) => { continue; }
                    Err(err)                           => { return Err(err); }
                };

                // Allocate new memory on this block (which we assume succeeds)
                let mut new_pool: BlockPool = Rc::try_unwrap(BlockPool::new(self.device.clone(), new_block)).dunwrap().into_inner();
                let (memory, mut pointer): (vk::DeviceMemory, GpuPtr) = new_pool.allocate(reqs, props)?;

                // Set the pointer indices
                pointer.set_type_idx(u32::from(mem_type.index) as u8);
                pointer.set_pool_idx(mem_type.pools.len()      as u16);

                // Now add the pool internally and return the new allocation
                mem_type.pools.push(new_pool);
                return Ok((memory, pointer));
            }

            // 5. If not found, attempt to seek another memory type that we already know of
            continue;
        }

        // 6. No free memory on the device anymore :(
        Err(Error::OutOfMemoryError{ req_size: reqs.size })
    }

    /// Frees an allocated bit of memory.
    /// 
    /// Note that not all types of pools may actually do anything with this. A LinearPool, for example, might deallocate but will never re-use that memory until reset anyway.
    /// 
    /// # Arguments
    /// - `pointer`: The pointer to the block that was allocated.
    /// 
    /// # Panics
    /// This function may panic if the given pointer was never allocated with this pool.
    fn free(&mut self, pointer: GpuPtr) {
        let type_idx: usize = pointer.type_idx() as usize;
        let pool_idx: usize = pointer.pool_idx() as usize;

        // Do some sanity checking on the type & pool index
        if type_idx >= self.types.len()                 { panic!("The given pointer {:?} was not allocated in this MetaPool: no type '{}'", pointer, type_idx); }
        if pool_idx >= self.types[type_idx].pools.len() { panic!("The given pointer {:?} was not allocated in this MetaPool: no pool '{}' in type {}", pointer, pool_idx, type_idx); }

        // We can instantly go to the correct memory type / pool
        self.types[type_idx].pools[pool_idx].free(pointer.agnostic())
    }

    /// Resets the memory pool back to its initial, empty state.
    #[inline]
    fn reset(&mut self) {
        // Reset all pools
        for mem_type in &mut self.types {
            for pool in &mut mem_type.pools {
                pool.reset();
            }
        }
    }



    /// Returns the device of the pool.
    #[inline]
    fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the used space in the pool.
    fn size(&self) -> usize { self.size }

    /// Returns the total space in the pool.
    #[inline]
    fn capacity(&self) -> usize { self.capacity }
}
