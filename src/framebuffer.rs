//  FRAMEBUFFER.rs
//    by Lut99
// 
//  Created:
//    03 May 2022, 18:20:39
//  Last edited:
//    06 Aug 2022, 10:55:25
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a wrapper around a VkFramebuffer (which wraps around an
// 

use std::ptr;
use std::rc::Rc;

use ash::vk;

pub use crate::errors::FramebufferError as Error;
use crate::log_destroy;
use crate::auxillary::structs::Extent2D;
use crate::device::Device;
use crate::render_pass::RenderPass;
use crate::image;


/***** POPULATE FUNCTIONS *****/
/// Populates a new VkFramebufferCreateInfo struct.
/// 
/// # Arguments
/// - `render_pass`: The VkRenderPass to attach this framebuffer to.
/// - `attachments`: The list of VkImageViews to attach to this framebuffer.
/// - `width`: The width (in pixels) of the views attached to this framebuffer.
/// - `height`: The height (in pixels) of the views attached to this framebuffer.
#[inline]
fn populate_framebuffer_info(render_pass: vk::RenderPass, attachments: &Vec<vk::ImageView>, width: u32, height: u32) -> vk::FramebufferCreateInfo {
    vk::FramebufferCreateInfo {
        // Do the default stuff.
        s_type : vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::FramebufferCreateFlags::empty(),

        // Set the render pass for this framebuffer.
        render_pass,

        // Set the list of attachments for this framebuffer.
        attachment_count : attachments.len() as u32,
        p_attachments    : attachments.as_ptr(),

        // Set the extent and the number of layers (which we fix to 1) of each of the attached views.
        width,
        height,
        layers : 1,
    }
}





/***** LIBRARY *****/
/// The Framebuffer defines a wrapper around one or more image views to represent a single renderable target.
pub struct Framebuffer {
    /// The device where the Framebuffer lives.
    device      : Rc<Device>,
    /// The RenderPass where the Framebuffer is attached to.
    render_pass : Rc<RenderPass>,
    /// The ImageViews that live in this Framebuffer.
    attachments : Vec<Rc<image::View>>,

    /// The VkFramebuffer we wrap.
    framebuffer : vk::Framebuffer,
    /// The extent of this Framebuffer.
    extent      : Extent2D<u32>,
}

impl Framebuffer {
    /// Constructor for the Framebuffer.
    /// 
    /// # Arguments
    /// - `device`: The Device where the Framebuffer will live.
    /// - `render_pass`: The RenderPass where the Framebuffer will be bound to.
    /// - `attachments`: A list of ImageViews to attach to this Framebuffer.
    /// - `extent`: The Extent2D of the attachments of this Framebuffer.
    /// 
    /// # Returns
    /// A new Framebuffer instance on success.
    /// 
    /// # Errors
    /// This function errors if the underlying Vulkan backend does.
    pub fn new(device: Rc<Device>, render_pass: Rc<RenderPass>, attachments: Vec<Rc<image::View>>, extent: Extent2D<u32>) -> Result<Rc<Self>, Error> {
        // Cast the attachments to their Vulkan counterparts
        let vk_attachments: Vec<vk::ImageView> = attachments.iter().map(|att| att.vk()).collect();

        // Populate the create info for the Framebuffer
        let framebuffer_info = populate_framebuffer_info(render_pass.vk(), &vk_attachments, extent.w, extent.h);

        // Create the new framebuffer on the device
        let framebuffer = unsafe {
            match device.create_framebuffer(&framebuffer_info, None) {
                Ok(framebuffer) => framebuffer,
                Err(err)        => { return Err(Error::FramebufferCreateError{ err }); }
            }
        };

        // Store it and relevant dependencies into the struct and done
        Ok(Rc::new(Self {
            device,
            render_pass,
            attachments,

            extent,
            framebuffer,
        }))
    }



    /// Returns the parent device.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the render pass where this Framebuffer is bound.
    #[inline]
    pub fn render_pass(&self) -> &Rc<RenderPass> { &self.render_pass }

    /// Returns the ImageViews bound to this Framebuffer.
    #[inline]
    pub fn attachments(&self) -> &[Rc<image::View>] { &self.attachments }



    /// Returns the internal Vulkan VkFramebuffer.
    #[inline]
    pub fn vk(&self) -> vk::Framebuffer { self.framebuffer }

    /// Returns the extent of this Framebuffer
    #[inline]
    pub fn extent(&self) -> &Extent2D<u32> { &self.extent }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        log_destroy!(self, Framebuffer);
        unsafe { self.device.destroy_framebuffer(self.framebuffer, None); }
    }
}
