//  PIPELINE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2022, 17:26:39
//  Last edited:
//    06 Aug 2022, 11:36:42
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a wrapper around the Vulkan pipeline.
// 

use std::ffi::{c_void, CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::Rc;

use ash::vk;

use crate::{debug, warn};
pub use crate::errors::PipelineError as Error;
use crate::log_destroy;
use crate::auxillary::enums::{BlendFactor, BlendOp, CompareOp, DynamicState, LogicOp, StencilOp, VertexTopology};
use crate::auxillary::flags::{ColourComponentFlags, ShaderStage};
use crate::auxillary::structs::{AttachmentBlendState, ColourBlendState, DepthTestingState, MultisampleState, RasterizerState,  StencilOpState, VertexAssemblyState, VertexInputState, ViewportState};
use crate::device::Device;
use crate::shader::{Error as ShaderError, Shader};
use crate::layout::PipelineLayout;
use crate::render_pass::RenderPass;


/***** POPULATE FUNCTIONS ******/
/// Populates a VkPipelineCacheCreateInfo struct.
/// 
/// # Arguments
/// - `data`: The raw binary of cache data that has been read from a previous run.
fn populate_cache_info(data: &[u8]) -> vk::PipelineCacheCreateInfo {
    vk::PipelineCacheCreateInfo {
        // Do the default stuff
        s_type : vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::PipelineCacheCreateFlags::empty(),

        // Set the previous cache data
        initial_data_size : data.len(),
        p_initial_data    : if data.len() > 0 { data.as_ptr() as *const c_void } else { ptr::null() },
    }
}

/// Populates a VkPipelineShaderStageCreateInfo struct with the shader data given.
/// 
/// # Arguments
/// - `entry`: The CStr that defines the name of the entry function in the shader (anything other than 'main' does not work :( ).
/// - `stage`: The VkShaderStage that determines where this shader will be run.
/// - `module`: The VkShaderModule that contains the shader code.
fn populate_shader_stage_info(entry: &CStr, stage: vk::ShaderStageFlags, module: vk::ShaderModule) -> vk::PipelineShaderStageCreateInfo {
    vk::PipelineShaderStageCreateInfo {
        // Set the default stuff
        s_type : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::PipelineShaderStageCreateFlags::empty(),

        // Set the shader information
        p_name : entry.as_ptr(),
        module,
        stage,

        // Set the specialization information for this shader (ignored for now)
        p_specialization_info : ptr::null(),
    }
}

/// Populates the given VkGraphicsPipelineCreateInfo struct with the configuration structs given.
/// 
/// # Arguments
/// - `base_pipeline`: A base Pipeline to (potentially) speed up building this one.
/// - `shader_stages`: The list of shader (stages) to enable for this pipeline.
/// - `vertex_input`: The information about the vertex layout for this pipeline.
/// - `vertex_assembly`: The information about the vertex list layout for this pipeline.
/// - `viewport`: The information about the resulting frame for this pipeline.
/// - `rasterizer`: The information about the rasterization stage of the pipeline.
/// - `multisampling`: The information about multisampling in the pipeline.
/// - `depth_testing`: The information about depth testing in the pipeline.
/// - `colour_blend`: The information about how to write fragments in the pipeline.
/// - `layout`: The PipelineLayout to base the pipeline on.
/// - `render_pass`: The RenderPass to base the pipeline on.
/// - `subpass`: The index of the first subpass in the render pass to run.
#[inline]
fn populate_graphics_pipeline_info(
    base_pipeline: vk::Pipeline,
    shader_stages: &Vec<vk::PipelineShaderStageCreateInfo>,
    vertex_input: &vk::PipelineVertexInputStateCreateInfo,
    vertex_assembly: &vk::PipelineInputAssemblyStateCreateInfo,
    viewport: &vk::PipelineViewportStateCreateInfo,
    rasterizer: &vk::PipelineRasterizationStateCreateInfo,
    multisampling: &vk::PipelineMultisampleStateCreateInfo,
    depth_testing: &vk::PipelineDepthStencilStateCreateInfo,
    colour_blend: &vk::PipelineColorBlendStateCreateInfo,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    subpass: u32,
) -> vk::GraphicsPipelineCreateInfo {
    vk::GraphicsPipelineCreateInfo {
        // Do the default stuff
        s_type : vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::PipelineCreateFlags::empty(),

        // Set the shaders
        stage_count : shader_stages.len() as u32,
        p_stages    : shader_stages.as_ptr(),

        // Set the fixed-function stuff
        p_vertex_input_state   : &*vertex_input,
        p_input_assembly_state : &*vertex_assembly,
        p_tessellation_state   : ptr::null(),
        p_viewport_state       : &*viewport,
        p_rasterization_state  : &*rasterizer,
        p_multisample_state    : &*multisampling,
        p_depth_stencil_state  : &*depth_testing,
        p_color_blend_state    : &*colour_blend,
        p_dynamic_state        : ptr::null(),

        // Set the layout and the render pass
        layout,
        render_pass,
        subpass,

        // Set the base pipeline handle and/or index
        base_pipeline_handle : base_pipeline,
        base_pipeline_index  : -1,
    }
}





/***** LIBRARY *****/
/// May speed up pipeline construction by caching the results and re-using that when possible.
pub struct PipelineCache {
    /// The parent Device of this PipelineCache.
    device : Rc<Device>,
    /// The path where the cache has to be written to when destroyed.
    path   : PathBuf,
    /// The underlying VkPipelineCache struct.
    cache  : vk::PipelineCache,
}

impl PipelineCache {
    /// Constructor for the PipelineCache.
    /// 
    /// # Generic types
    /// - `P`: The Path-like type of the path.
    /// 
    /// # Arguments
    /// - `device`: The Device where the cache will live.
    /// - `path`: Path to the cache file. If it does not exist, initializes the pipeline caches as empty. Once done, the cache will be written back to this path for future use.
    /// 
    /// # Errors
    /// This function errors if the Vulkan backend could not create the new cache, or the given file existed but could not be read.
    pub fn new<P: AsRef<Path>>(device: Rc<Device>, path: P) -> Result<Rc<Self>, Error> {
        // Convert Path-likes to Path
        let path: &Path = path.as_ref();

        // Try to read the cache file
        let mut data: Vec<u8> = Vec::new();
        match File::open(path) {
            Ok(mut handle) => {
                // Try to read the file into the data buffer
                if let Err(err) = handle.read_to_end(&mut data) {
                    return Err(Error::PipelineCacheReadError{ path: path.to_path_buf(), err });
                }
            },
            Err(err) => {
                // Only actually error if not exists
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(Error::PipelineCacheOpenError{ path: path.to_path_buf(), err });
                }
            }
        };

        // Create the create info with this data
        let cache_info = populate_cache_info(&data);

        // Create the pipeline cache with that
        let cache = unsafe {
            match device.create_pipeline_cache(&cache_info, None) {
                Ok(cache) => cache,
                Err(err)  => { return Err(Error::PipelineCacheCreateError{ err }); }
            }  
        };

        // Done, wrap it in a struct and return
        debug!("Loaded pipeline cache from '{}'", path.display());
        Ok(Rc::new(Self {
            device,
            path : path.to_path_buf(),
            cache,
        }))
    }



    /// Returns the parent Device.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the underlying VkPipelineCache struct.
    #[inline]
    pub fn vk(&self) -> vk::PipelineCache { self.cache }
}

impl Drop for PipelineCache {
    fn drop(&mut self) {
        // Try to save the cache to a file
        log_destroy!(self, PipelineCache);
        match unsafe { self.device.get_pipeline_cache_data(self.cache) } {
            Ok(data) => {
                // Try to open the file
                match File::create(&self.path) {
                    Ok(mut handle) => {
                        // Try to write to it
                        match handle.write(&data) {
                            Ok(_) => { debug!("Written pipeline cache to '{}'", self.path.display()); }
                            #[allow(unused_variables)]
                            Err(err) => {
                                warn!("Could not write to pipeline cache file '{}': {}", self.path.display(), err);
                                warn!("Pipeline cache not saved");
                            }
                        }
                    },
                    #[allow(unused_variables)]
                    Err(err) => {
                        warn!("Could not create pipeline cache file '{}': {}", self.path.display(), err);
                        warn!("Pipeline cache not saved");
                    }
                }
            },
            #[allow(unused_variables)]
            Err(err) => {
                warn!("Could not get pipeline cache data: {}", err);
                warn!("Pipeline cache not saved");
            }
        }

        // With the cache (hopefully) written, destroy it
        unsafe { self.device.destroy_pipeline_cache(self.cache, None); }
    }
}



/// Extended constructor for the Pipeline that may be used to configure it.
pub struct PipelineBuilder {
    /// Collects errors until build() gets called.
    error  : Option<Error>,

    // Caching stuff
    /// An optional PipelineCache to build from during pipeline creation.
    cache         : Option<Rc<PipelineCache>>,
    /// An optional base pipeline to start construction from.
    base_pipeline : Option<Rc<Pipeline>>,
    
    // Default stuff
    /// Describes how we treat the input vertices.
    vertex_assembly : VertexAssemblyState,
    /// Describes the multisample stage
    multisampling   : MultisampleState,
    /// Describes if and how depth testing is done
    depth_testing   : DepthTestingState,
    /// Describes how to write colours to the output frame
    colour_blending : ColourBlendState,
    /// Defines any dynamic part of the pipeline
    dynamic         : Vec<DynamicState>,

    // Non-default stuff
    /// Defines the different shaders used in this pipeline
    shaders       : Vec<(ShaderStage, Rc<Shader>)>,
    /// Describes how the input vertices look like.
    vertex_input  : Option<VertexInputState>,
    /// Describes the output images dimensions, cutoff and depth.
    viewport      : Option<ViewportState>,
    /// Describes the rasterization stage
    rasterization : Option<RasterizerState>,
}

impl PipelineBuilder {
    /// Constructor for the PipelineBuilder.
    /// 
    /// Spawns a new PipelineBuilder with, where possible, default settings.
    /// 
    /// Use the other functions to configure the pipeline. When done, call `PipelineBuilder::build()` to get the Pipeline. Any errors that occur mid-build will be propagated until that function.
    #[inline]
    pub fn new() -> Self {
        debug!("Starting Pipeline construction");
        Self {
            error : None,

            cache         : None,
            base_pipeline : None,

            vertex_assembly : VertexAssemblyState {
                topology          : VertexTopology::TriangleList,
                restart_primitive : false,
            },
            multisampling : MultisampleState {},
            depth_testing : DepthTestingState {
                enable_depth   : false,
                enable_write   : false,
                enable_stencil : false,
                enable_bounds  : false,

                compare_op : CompareOp::LessEq,
                
                pre_stencil_test : StencilOpState {
                    on_stencil_fail : StencilOp::Keep,
                    on_depth_fail   : StencilOp::Keep,
                    on_success      : StencilOp::Keep,

                    compare_op   : CompareOp::Always,
                    compare_mask : 0,
                    write_mask   : 0,
                    reference    : 0,  
                },
                post_stencil_test : StencilOpState {
                    on_stencil_fail : StencilOp::Keep,
                    on_depth_fail   : StencilOp::Keep,
                    on_success      : StencilOp::Keep,

                    compare_op   : CompareOp::Always,
                    compare_mask : 0,
                    write_mask   : 0,
                    reference    : 0,  
                },

                min_bound : 1.0,
                max_bound : 0.0,
            },
            colour_blending : ColourBlendState {
                enable_logic : false,
                logic_op     : LogicOp::Copy,

                attachment_states : vec![AttachmentBlendState {
                    enable_blend : false,
                    
                    src_colour : BlendFactor::One,
                    dst_colour : BlendFactor::Zero,
                    colour_op  : BlendOp::Add,

                    src_alpha : BlendFactor::One,
                    dst_alpha : BlendFactor::Zero,
                    alpha_op  : BlendOp::Add,

                    write_mask : ColourComponentFlags::all(),
                }],
                blend_constants: [0.0, 0.0, 0.0, 0.0],
            },
            dynamic : vec![],

            shaders       : Vec::with_capacity(2),
            vertex_input  : None,
            viewport      : None,
            rasterization : None,
        }
    }



    /// Uses the given PipelineCache as a pool to create new pipelines with.
    /// 
    /// # Arguments
    /// - `cache`: The PipelineCache to cache new pipelines in, and to possibly speedup building pipelines we build before.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn set_cache(mut self, cache: Rc<PipelineCache>) -> Self {
        if self.error.is_some() { return self; }
        
        // Simply set the cache
        self.cache = Some(cache);

        // Done
        debug!("Registered pipeline cache");
        self
    }

    /// Uses the given PipelineCache as a pool to create new pipelines with (given as a result from a constructor call).
    /// 
    /// # Arguments
    /// - `cache`: The PipelineCache to cache new pipelines in, and to possibly speedup building pipelines we build before.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn try_cache(mut self, cache: Result<Rc<PipelineCache>, Error>) -> Self {
        if self.error.is_some() { return self; }

        // Unpack the cache
        let cache = match cache {
            Ok(cache) => cache,
            Err(err)  => {
                // Set as error and immediately quit
                self.error = Some(Error::PipelineCacheError{ err: Box::new(err) });
                return self;
            }
        };

        // Simply set the cache
        self.cache = Some(cache);

        // Done
        debug!("Registered pipeline cache");
        self
    }

    /// Uses the given pipeline as a base for constructing the new one.
    /// 
    /// # Arguments
    /// - `pipeline`: The Pipeline to base this new one off.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn set_pipeline(mut self, pipeline: Rc<Pipeline>) -> Self {
        if self.error.is_some() { return self; }
        
        // Simply set the pipeline
        self.base_pipeline = Some(pipeline);

        // Done
        debug!("Registered base pipeline");
        self
    }



    /// Adds a certain Shader to the pipeline.
    /// 
    /// You should probably define a shader for at least the vertex and fragment stages.
    /// 
    /// # Arguments
    /// - `stage`: The ShaderStage where the Shader will be ran.
    /// - `shader`: The Shader to add to the Pipeline.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn shader(mut self, stage: ShaderStage, shader: Rc<Shader>) -> Self {
        if self.error.is_some() { return self; }

        // Add the shader internally
        self.shaders.push((stage, shader));

        // Done, return ourselves again
        debug!("Defined {} Shader", stage);
        self
    }

    /// ATries to add a certain Shader to the pipeline directly after its constructor call.
    /// 
    /// Errors if the call fails (though it propagates this to `PipelineBuilder::build()`).
    /// 
    /// You should probably define a shader for at least the vertex and fragment stages.
    /// 
    /// # Arguments
    /// - `stage`: The ShaderStage where the Shader will be ran.
    /// - `shader`: The result of the Shader constructor call to add to the Pipeline.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn try_shader(mut self, stage: ShaderStage, shader: Result<Rc<Shader>, ShaderError>) -> Self {
        if self.error.is_some() { return self; }

        // Try to unpack the shader
        let shader = match shader {
            Ok(shader) => shader,
            Err(err)   => {
                self.error = Some(Error::ShaderError{ err });
                return self;
            }
        };

        // Add the shader internally
        self.shaders.push((stage, shader));

        // Done, return ourselves again
        debug!("Defined {} Shader", stage);
        self
    }

    /// Define a VertexInputState for this Pipeline.
    /// 
    /// This is one of the non-default functions that must always be called to define the input.
    /// 
    /// # Arguments
    /// - `info`: The new VertexInputState struct that describes how the input vertices look like.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn vertex_input(mut self, info: VertexInputState) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.vertex_input = Some(info);

        // Done, return us again
        debug!("Defined vertex input state");
        self
    }

    /// Define a non-default VertexAssemblyState for this Pipeline.
    /// 
    /// By default, the Pipeline uses a VertexAssemblyState with:
    /// - A triangle topology.
    /// - No primitive restart enabled (i.e., special value that allows mid-triangle to reset).
    /// 
    /// # Arguments
    /// - `info`: The new VertexAssemblyState struct that describes what to do with the input vertices.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn vertex_assembly(mut self, info: VertexAssemblyState) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.vertex_assembly = info;

        // Done, return us again
        debug!("Defined non-default vertex assembly state");
        self
    }

    /// Defines how the viewport looks like, i.e., the size of the output frame.
    /// 
    /// This is one of the non-default functions that must always be called to define the input.
    /// 
    /// # Arguments
    /// - `info`: The new Viewport struct that describes how the output frame looks like.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn viewport(mut self, info: ViewportState) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.viewport = Some(info);

        // Done, return us again
        debug!("Defined viewport state");
        self
    }

    /// Defines the configuration of the rasterization stage.
    /// 
    /// This is one of the non-default functions that must always be called to define the input.
    /// 
    /// # Arguments
    /// - `info`: The new Rasterization struct that describes the config.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn rasterization(mut self, info: RasterizerState) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.rasterization = Some(info);

        // Done, return us again
        debug!("Defined rasterization state");
        self
    }

    /// Define a non-default configuration of how to multisample.
    /// 
    /// By default, no multisampling is used.
    /// 
    /// # Arguments
    /// - `info`: The new Multisampling struct that describes the config.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn multisampling(self, _info: MultisampleState) -> Self {
        if self.error.is_some() { return self; }

        warn!("Called useless PipelineBuilder::multisampling() function");
        self
    }

    /// Define a non-default configuration of if and how to perform depth testing.
    /// 
    /// By default, no depth testing is used.
    /// 
    /// # Arguments
    /// - `info`: The new DepthTestingState struct that describes the config.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn depth_testing(mut self, info: DepthTestingState) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.depth_testing = info;

        // Done, return us again
        debug!("Defined non-default depth testing state");
        self
    }

    /// Define a non-default configuration of how to multisample.
    /// 
    /// By default, the source colour fragments are always copied over the destination ones already present in the frame.
    /// 
    /// # Arguments
    /// - `info`: The new ColourBlendState struct that describes the config.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn colour_blending(mut self, info: ColourBlendState) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.colour_blending = info;

        // Done, return us again
        debug!("Defined non-default colour blending state");
        self
    }

    /// Defines the parts of the Pipeline that will be dynamic.
    /// 
    /// By default, no dynamic state is defined.
    /// 
    /// # Arguments
    /// - `info`: A list of Pipeline parts (as DynamicStates) to make dynamic.
    /// 
    /// # Returns
    /// Because this function is consuming, returns the same instance of self as passed to it.
    /// 
    /// # Errors
    /// This function doesn't error directly, but may pass any incoming errors to the `PipelineBuilder::build()` call.
    pub fn dynamic_state(mut self, states: Vec<DynamicState>) -> Self {
        if self.error.is_some() { return self; }

        // Set the state
        self.dynamic = states;

        // Done, return us again
        debug!("Defined non-default dynamic state");
        self
    }



    /// Builds the Pipeline as a Graphics pipeline, requiring at least the following functions:
    /// - `PipelineBuidler::shader()` or `PipelineBuilder::try_shader()`, for a Vertex shader.
    /// - `PipelineBuidler::shader()` or `PipelineBuilder::try_shader()`, for a Fragment shader.
    /// - `PipelineBuilder::vertex_input()`.
    /// - `PipelineBuilder::viewport()`.
    /// - `PipelineBuilder::rasterization()`.
    /// 
    /// After the build is complete, you can use this builder to generate more pipelines. Those subsequent pipelines will use this pipeline as their base (unless `PipelineBuilder::set_pipeline()` is called to override it).
    /// 
    /// # Arguments
    /// - `device`: The Device where the pipeline will live and be build for.
    /// - `layout`: The PipelineLayout that defines the resources that will be present in this Pipeline.
    /// - `render_pass`: Describes the configurable process for this pipeline.
    /// 
    /// # Returns
    /// A new Pipeline on success.
    /// 
    /// # Errors
    /// This function returns an error if the backend Vulkan driver errors while creating the pipeline, or if an error occurred during any of the other functions.
    pub fn build(&mut self, device: Rc<Device>, layout: Rc<PipelineLayout>, render_pass: Rc<RenderPass>) -> Result<Rc<Pipeline>, Error> {
        let Self { ref base_pipeline, ref shaders, ref vertex_input, ref vertex_assembly, ref viewport, ref rasterization, ref multisampling, ref depth_testing, ref colour_blending, .. } = self;

        // First, cast the stages and shaders to VkShaderStageFlags and VkShaderModules
        let entry_point = CString::new("main").unwrap();
        let vk_shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = shaders.iter().map(|(stage, shader)| populate_shader_stage_info(&entry_point, stage.into(), shader.vk())).collect();

        // Next, cast the vertex input & assemply info
        let (vk_vertex_input, _vk_vertex_input_mem): (vk::PipelineVertexInputStateCreateInfo, (Vec<vk::VertexInputAttributeDescription>, Vec<vk::VertexInputBindingDescription>)) = vertex_input.as_ref().expect("Called PipelineBuilder::build() without calling PipelineBuilder::vertex_input()").clone().into();
        let vk_vertex_assembly: vk::PipelineInputAssemblyStateCreateInfo = vertex_assembly.clone().into();

        // Then, cast the Viewport
        let (vk_viewport, _vk_viewport_mem): (vk::PipelineViewportStateCreateInfo, (Box<vk::Viewport>, Box<vk::Rect2D>)) = viewport.as_ref().expect("Called PipelineBuilder::build() without calling PipelineBuilder::viewport()").clone().into();

        // Cast the rasterizer & multisampling states
        let vk_rasterizer: vk::PipelineRasterizationStateCreateInfo = rasterization.as_ref().expect("Called PipelineBuilder::build() without calling PipelineBuilder::rasterization()").clone().into();
        let vk_multisampling: vk::PipelineMultisampleStateCreateInfo = multisampling.clone().into();

        // Cast the depth & colour attachment states
        let vk_depth_testing: vk::PipelineDepthStencilStateCreateInfo = depth_testing.clone().into();
        let (vk_colour_blend, _vk_colour_blend_mem): (vk::PipelineColorBlendStateCreateInfo, Vec<vk::PipelineColorBlendAttachmentState>) = colour_blending.clone().into();

        // Now populate the struct
        let pipeline_info = populate_graphics_pipeline_info(
            base_pipeline.as_ref().map(|pipeline| pipeline.vk()).unwrap_or(vk::Pipeline::null()),
            &vk_shader_stages,
            &vk_vertex_input,
            &vk_vertex_assembly,
            &vk_viewport,
            &vk_rasterizer,
            &vk_multisampling,
            &vk_depth_testing,
            &vk_colour_blend,
            layout.vk(),
            render_pass.vk(),
            0
        );

        // With that, create the pipeline...
        let pipeline = unsafe {
            match device.create_graphics_pipelines(self.cache.as_ref().map(|cache| cache.vk()).unwrap_or(vk::PipelineCache::null()), &[pipeline_info], None) {
                Ok(pipelines) => {
                    // Return the first
                    pipelines[0]
                },
                Err((_, err)) => { return Err(Error::PipelineCreateError{ err }); }
            }
        };

        // Wrap it in a Pipeline struct, set it as the base for subsequent calls and return it
        let pipeline = Rc::new(Pipeline {
            device,
            layout,
            render_pass,

            pipeline,
        });
        self.base_pipeline = Some(pipeline.clone());
        debug!("Successfully built Pipeline");
        Ok(pipeline)
    }
}



/// Wraps around a Vulkan Pipeline, which describes the process of rendering some vertices to an image.
pub struct Pipeline {
    /// The parent device of this pipeline.
    device      : Rc<Device>,
    /// The layout for this Pipeline.
    layout      : Rc<PipelineLayout>,
    /// The render pass for this Pipeline.
    render_pass : Rc<RenderPass>,

    /// The VkPipeline that we wrap around.
    pipeline : vk::Pipeline,
}

impl Pipeline {
    /// Returns the parent device of this pipeline.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }

    /// Returns the layout of this pipeline.
    #[inline]
    pub fn layout(&self) -> &Rc<PipelineLayout> { &self.layout }

    /// Returns the render pass of this pipeline.
    #[inline]
    pub fn render_pass(&self) -> &Rc<RenderPass> { &self.render_pass }



    /// Returns the VkPipeline behind this pipeline.
    #[inline]
    pub fn vk(&self) -> vk::Pipeline { self.pipeline }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        log_destroy!(self, Pipeline);
        unsafe { self.device.destroy_pipeline(self.pipeline, None); }
    }
}
