//  VIEW.rs
//    by Lut99
// 
//  Created:
//    05 Apr 2022, 17:41:18
//  Last edited:
//    06 Aug 2022, 10:50:55
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains code related to image views.
// 

use std::ptr;
use std::rc::Rc;

use ash::vk;

// pub use crate::errors::ImageError;
pub use crate::errors::ImageViewError as Error;
use crate::auxillary::enums::{ImageFormat, ImageAspect, ImageViewKind};
use crate::auxillary::structs::ComponentMapping;
use crate::device::Device;
use crate::image::Image;


/***** AUXILLARY STRUCTS *****/
/// CreateInfo for the View.
#[derive(Clone, Debug)]
pub struct CreateInfo {
    /// Defines the type of the image view
    pub kind    : ImageViewKind,
    /// Defines the format of the image
    pub format  : ImageFormat,
    /// Defines the channel mapping for the image
    pub swizzle : ComponentMapping,

    /// Defines the aspect for this image (how it will be used)
    pub aspect     : ImageAspect,
    /// Defines the base MIP level
    pub base_level : u32,
    /// Defines the number of image MIP levels
    pub mip_levels : u32,
}

impl Default for CreateInfo {
    #[inline]
    fn default() -> Self {
        Self {
            kind    : ImageViewKind::TwoD,
            format  : ImageFormat::B8G8R8A8SRgb,
            swizzle : ComponentMapping::default(),

            aspect     : ImageAspect::Colour,
            base_level : 0,
            mip_levels : 1,
        }
    }
}





/***** LIBRARY *****/
/// The ImageView class, which wraps around an Image or a VkImage to define how it should be accessed.
pub struct View {
    /// The parent device for the parent image, who's lifetime we are tied  to
    device : Rc<Device>,
    /// The parent image for this view
    image  : Rc<Image>,

    /// The image view object itself.
    view  : vk::ImageView,
}

impl View {
    /// Constructor for the View.
    /// 
    /// Creates a new ImageView with the given properties from the given Image.
    /// 
    /// # Arguments
    /// - `device`: The Device to allocate this view on.
    /// - `image`: The Image to base this view on.
    /// - `create_info`: The CreateInfo with additional properties to set in this View that are not necessarily deriveable from the image itself.
    /// 
    /// # Returns
    /// A new View instance.
    /// 
    /// # Errors
    /// This function errors if we failed to allocate the new ImageView for some reason.
    pub fn new(device: Rc<Device>, image: Rc<Image>, create_info: CreateInfo) -> Result<Rc<Self>, Error> {
        // Define the Vulkan create info
        let image_info = vk::ImageViewCreateInfo {
            // Do the default stuff
            s_type : vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::ImageViewCreateFlags::empty(),

            // Define the type of the image
            view_type  : create_info.kind.into(),
            // Define the format of the image
            format     : create_info.format.into(),
            // Define the component swizzler
            components : create_info.swizzle.into(),

            // Populate the subresource range
            subresource_range : vk::ImageSubresourceRange {
                aspect_mask      : create_info.aspect.into(),
                base_mip_level   : create_info.base_level,
                level_count      : create_info.mip_levels,
                base_array_layer : 0,
                layer_count      : 1,
            },

            // Finally, set the image
            image : image.vk(),
        };

        // Use that to create the view
        let view = unsafe {
            match device.create_image_view(&image_info, None) {
                Ok(view) => view,
                Err(err) => { return Err(Error::ViewCreateError{ err }); }
            }
        };

        // Return the new instance
        Ok(Rc::new(Self {
            device,
            image,

            view,
        }))
    }

    // /// Constructor for the View, from a VkImage instead of a Rusty one.
    // /// 
    // /// # Arguments
    // /// - `gpu`: The GPU to allocate the view on.
    // /// - `image`: The VkImage to base this image on.
    // /// - `create_info`: The CreateInfo for this image view.
    // /// 
    // /// # Returns
    // /// The new View instance on success, or else an Error.
    // pub fn from_vk(device: Rc<Device>, image: vk::Image, create_info: CreateInfo) -> Result<Self, Error> {
    //     // Define the create info
    //     let image_info = vk::ImageViewCreateInfo {
    //         // Do the default stuff
    //         s_type : vk::StructureType::IMAGE_VIEW_CREATE_INFO,
    //         p_next : ptr::null(),
    //         flags  : vk::ImageViewCreateFlags::empty(),
            
    //         // Define the type of the image
    //         view_type  : create_info.kind,
    //         // Define the format of the image
    //         format     : create_info.format,
    //         // Define the component swizzler
    //         components : create_info.swizzle.into(),

    //         // Populate the subresource range
    //         subresource_range : vk::ImageSubresourceRange {
    //             aspect_mask      : create_info.aspect,
    //             base_mip_level   : create_info.base_level,
    //             level_count      : create_info.mip_levels,
    //             base_array_layer : 0,
    //             layer_count      : 1,
    //         },

    //         // Finally, set the image
    //         image,
    //     };

    //     // Use that to create the view
    //     let view = unsafe {
    //         match gpu.create_image_view(&image_info, None) {
    //             Ok(view) => view,
    //             Err(err) => { return Err(Error::ViewCreateError{ err }); }
    //         }
    //     };

    //     // Return the new instance
    //     Ok(Self {
    //         gpu,
    //         image,
    //         view,
    //     })
    // }



    /// Returns a reference to the parent GPU
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns a reference to the parent image
    #[inline]
    pub fn image(&self) -> &Rc<Image> { &self.image }



    /// Returns a reference to the internal view
    #[inline]
    pub fn vk(&self) -> vk::ImageView { self.view }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe { self.device.destroy_image_view(self.view, None); };
    }
}
