//  DESCRIPTORS.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2022, 11:57:55
//  Last edited:
//    06 Aug 2022, 10:54:49
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains the definitions for a DescriptorSet and a
// 

use std::ptr;
use std::rc::Rc;

use ash::vk;

pub use crate::errors::DescriptorError as Error;
use crate::log_destroy;
use crate::auxillary::structs::DescriptorBinding;
use crate::device::Device;


/***** POPULATE FUNCTIONS *****/
/// Populates a new VkDescriptorSetLayoutCreateInfo struct with the given parameters.
/// 
/// # Arguments
/// - `bindings`: The list of VkDescriptorSetLayoutBindings to attach to the create info.
/// 
/// # Returns
/// A new VkDescriptorSetLayoutCreateInfo struct with the same lifetime as the given reference.
#[inline]
fn populate_layout_info(bindings: &[vk::DescriptorSetLayoutBinding]) -> vk::DescriptorSetLayoutCreateInfo {
    vk::DescriptorSetLayoutCreateInfo {
        // Set the default stuff
        s_type : vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::DescriptorSetLayoutCreateFlags::empty(),

        // Attach the bindings
        p_bindings    : bindings.as_ptr(),
        binding_count : bindings.len() as u32,
    }
}





/***** LIBRARY *****/
/// Defines the DescriptorSetLayout, which describes one type of resource in the pipeline.
pub struct DescriptorSetLayout {
    /// The parent device for this layout.
    device : Rc<Device>,
    /// The VkDescriptorSetLayout itself.
    layout : vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    /// Constructor for the DescriptorSetLayout.
    /// 
    /// # Arguments
    /// - `device`: The parent device for this layout.
    /// - `bindings`: Each of the bindings for this set.
    /// 
    /// # Returns
    /// A new DescriptorSetLayout on success.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend failed to create a new DescriptorSetLayout.
    pub fn new(device: Rc<Device>, bindings: &[DescriptorBinding]) -> Result<Rc<Self>, Error> {
        // Cast the bindings to their Vulkan counterparts.
        let bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings.iter().map(|binding| binding.into()).collect();

        // Populate the create info based on the bindings.
        let layout_info = populate_layout_info(&bindings);

        // Create the layout with that
        let layout = unsafe {
            match device.create_descriptor_set_layout(&layout_info, None) {
                Ok(layout) => layout,
                Err(err)   => { return Err(Error::DescriptorSetLayoutCreateError{ err }); }
            }
        };

        // Return it wrapped in the struct
        Ok(Rc::new(Self {
            device,
            layout,
        }))
    }



    /// Returns the parent device of this DescriptorSetLayout.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the underlying VkDescriptorSetLayout struct.
    #[inline]
    pub fn vk(&self) -> vk::DescriptorSetLayout { self.layout }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        log_destroy!(self, DescriptorSetLayout);
        unsafe { self.device.destroy_descriptor_set_layout(self.layout, None); }
    }
}



/// Defines the DescriptorSet, which describes one resource in the pipeline.
pub struct DescriptorSet {
    
}
