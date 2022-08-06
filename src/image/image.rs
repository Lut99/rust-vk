//  IMAGE.rs
//    by Lut99
// 
//  Created:
//    18 Apr 2022, 14:34:47
//  Last edited:
//    06 Aug 2022, 10:50:47
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a wrapper around Vulkan's Image buffer.
// 

use std::rc::Rc;

use ash::vk;

pub use crate::errors::ImageError as Error;


/***** LIBRARY *****/
/// Represents an image, which is a kind of buffer that we may render to.
pub struct Image {
    /// The VkImage we wrap around.
    image : vk::Image,
}

impl Image {
    /// Constructor for the Image, which takes an already existing VkImage and wraps around it.
    pub(crate) fn from_vk(image: vk::Image) -> Result<Rc<Self>, Error> {
        Ok(Rc::new(Self {
            image,
        }))
    }



    /// Returns the internal VkImage.
    #[inline]
    pub fn vk(&self) -> vk::Image { self.image }
}
