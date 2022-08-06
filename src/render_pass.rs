//  RENDER PASS.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2022, 17:57:08
//  Last edited:
//    06 Aug 2022, 11:06:31
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a RenderPass for use in pipelines.
// 

use std::ptr;
use std::rc::Rc;

use ash::vk;

use crate::debug;
pub use crate::errors::RenderPassError as Error;
use crate::log_destroy;
use crate::auxillary::structs::{AttachmentDescription, SubpassDependency, SubpassDescription};
use crate::device::Device;


/***** POPULATE FUNCTIONS *****/
/// Populates the given VkRenderPassCreateInfo struct.
/// 
/// # Arguments
/// - `attachments`: The list of attachment descriptions for this RenderPass.
/// - `subpasses`: The list of subpasses for this RenderPass.
/// - `dependencies`: The list subpass dependencies for this RenderPass.
#[inline]
fn populate_render_pass_info(attachments: &Vec<vk::AttachmentDescription>, subpasses: &Vec<vk::SubpassDescription>, dependencies: &Vec<vk::SubpassDependency>) -> vk::RenderPassCreateInfo {
    vk::RenderPassCreateInfo {
        // Do the default stuff
        s_type : vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::RenderPassCreateFlags::empty(),

        // Set the attachments
        attachment_count : attachments.len() as u32,
        p_attachments    : attachments.as_ptr(),

        // Set the subpasses
        subpass_count : subpasses.len() as u32,
        p_subpasses   : subpasses.as_ptr(),

        // Set the attachments
        dependency_count : dependencies.len() as u32,
        p_dependencies   : dependencies.as_ptr(),
    }
}





/***** LIBRARY *****/
/// Defines a builder for a render pass.
pub struct RenderPassBuilder {
    /// Collects errors until build() gets called.
    error  : Option<Error>,

    /// The list of attachments for this RenderPass.
    attachments  : Vec<AttachmentDescription>,
    /// The list of subpasses for this RenderPass.
    subpasses    : Vec<SubpassDescription>,
    /// The list of inter-subpass dependencies for this RenderPass.
    dependencies : Vec<SubpassDependency>,
}

impl RenderPassBuilder {
    /// Constructor for the RenderPassBuilder.
    /// 
    /// Spawns a new RenderPassBuilder with no attachments or subpasses whatsoever.
    /// 
    /// Use other function calls to add these. When done, call `RenderPassBuilder::build()` to get the RenderPass. Any errors that occur mid-build will be propagated until that function.
    #[inline]
    pub fn new() -> Self {
        debug!("Starting RenderPass construction");
        Self {
            error : None,

            attachments  : Vec::with_capacity(3),
            subpasses    : Vec::with_capacity(1),
            dependencies : vec![],
        }
    }



    /// Defines a new attachment for this RenderPass.
    /// 
    /// # Arguments
    /// - `index`: If not None, an unsigned integer that will contain the index of the new attachment when ready. This number will simply be 0 for the first attachment, and then increments on every call.
    /// - `attachment`: The attachment to, well, attach.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `RenderPassBuilder::build()` call.
    pub fn attachment(mut self, index: Option<&mut u32>, attachment: AttachmentDescription) -> Self {
        if self.error.is_some() { return self; }

        // Get the index if requested
        if let Some(index) = index {
            *index = self.attachments.len() as u32;
        }

        // Add the attachment to the internal list
        self.attachments.push(attachment);

        // Done, return
        debug!("Defined attachment");
        self
    }

    /// Defines a new subpass in the RenderPass.
    /// 
    /// # Arguments
    /// - `index`: If not None, an unsigned integer that will contain the index of the new subpass when ready. This number will simply be 0 for the first subpass, and then increments on every call.
    /// - `subpass`: The SubpassDescription that describes the new Subpass. You can reference attachments using the index of the `RenderPassBuilder::attachment()` call.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `RenderPassBuilder::build()` call.
    pub fn subpass(mut self, index: Option<&mut u32>, subpass: SubpassDescription) -> Self {
        if self.error.is_some() { return self; }

        // Get the index if requested
        if let Some(index) = index {
            *index = self.subpasses.len() as u32;
        }

        // Add the subpass to the internal list
        self.subpasses.push(subpass);

        // Done, return
        debug!("Defined subpass");
        self
    }

    /// Defines a new dependency between two subpasses.
    /// 
    /// # Arguments
    /// - `dependency`: The SubpassDependency that describes the dependency. You can reference subpasses using the index of the `RenderPassBuilder::subpass()` call.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `RenderPassBuilder::build()` call.
    pub fn dependency(mut self, dependency: SubpassDependency) -> Self {
        if self.error.is_some() { return self; }

        // Add the subpass to the internal list
        self.dependencies.push(dependency);

        // Done, return
        debug!("Defined subpass dependency");
        self
    }



    /// Builds a new RenderPass based on the given data.
    /// 
    /// # Arguments
    /// - `device`: The Device where to create the RenderPass on.
    /// 
    /// # Returns
    /// A new RenderPass on success.
    /// 
    /// # Errors
    /// Whenever the creation of the new VkRenderPass failed, or when an error occurred during any of the other functions during the build process.
    pub fn build(self, device: Rc<Device>) -> Result<Rc<RenderPass>, Error> {
        // If any errors, then return those
        if let Some(err) = self.error { return Err(err); }

        // Cast the attachments to their Vulkan counterparts
        debug!("Casting attachments...");
        let attachments: Vec<vk::AttachmentDescription> = self.attachments.iter().map(|attach| attach.into()).collect();

        // Cast the subpasses (with associated memory) to Vulkan counterparts
        debug!("Casting subpasses...");
        let mut subpasses: Vec<vk::SubpassDescription> = Vec::with_capacity(self.subpasses.len());
        let mut _subpasses_mem: Vec<(Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<u32>, Option<Box<vk::AttachmentReference>>)> = Vec::with_capacity(self.subpasses.len());
        for subpass in self.subpasses {
            // Convert to Vulkan
            let result: (vk::SubpassDescription, (Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<u32>, Option<Box<vk::AttachmentReference>>)) = subpass.into();
            debug!("Depth stencil after into(): {:?}", if let Some(p) = result.1.4.as_ref() { &**p as *const vk::AttachmentReference } else { ptr::null() });

            // Store in the arrays
            subpasses.push(result.0);
            _subpasses_mem.push(result.1);
            debug!("Depth stencil after push(): {:?}", if let Some(p) = _subpasses_mem.last().unwrap().4.as_ref() { &**p as *const vk::AttachmentReference } else { ptr::null() });
        }

        // Cast the dependencies
        debug!("Casting dependencies...");
        let dependencies: Vec<vk::SubpassDependency> = self.dependencies.iter().map(|dep| dep.into()).collect();

        // Now populate the create info for the render pass with this
        debug!("Populating render pass info...");
        let render_pass_info = populate_render_pass_info(&attachments, &subpasses, &dependencies);
        debug!("Depth stencil according to render_pass: {:?}", unsafe { std::slice::from_raw_parts(render_pass_info.p_subpasses, render_pass_info.subpass_count as usize) }[0].p_depth_stencil_attachment);

        // Create the new RenderPass...
        let render_pass = unsafe {
            debug!("Creating VkRenderPass...");
            match device.create_render_pass(&render_pass_info, None) {
                Ok(render_pass) => render_pass,
                Err(err)        => { return Err(Error::RenderPassCreateError{ err }); }
            }
        };

        // Done! Wrap in the new struct and return
        debug!("Successfully built RenderPass");
        Ok(Rc::new(RenderPass {
            device,
            render_pass,
        }))
    }
}



/// Defines a render pass, i.e., a single run through a/the pipeline.
pub struct RenderPass {
    /// The device where the RenderPass will live.
    device : Rc<Device>,

    /// The Vulkan RenderPass which we wrap.
    render_pass : vk::RenderPass,
}

impl RenderPass {
    /// Returns the internal device in the RenderPass.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }
    
    /// Returns the internal VkRenderPass in the RenderPass.
    #[inline]
    pub fn vk(&self) -> vk::RenderPass { self.render_pass }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        log_destroy!(self, RenderPass);
        unsafe { self.device.destroy_render_pass(self.render_pass, None); }
    }
}
