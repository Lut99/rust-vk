//  STRUCTS.rs
//    by Lut99
// 
//  Created:
//    09 Jul 2022, 12:22:50
//  Last edited:
//    15 Aug 2022, 17:58:51
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements (non-flag) auxillary structs that represent various
//!   Vulkan
// 

use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter, Result as FResult};
use std::ops::Range;
use std::ptr;
use std::rc::Rc;
use std::slice;

use ash::vk;

use crate::errors::QueueError;
use crate::{to_cstring, vec_as_ptr};
use crate::spec::{ApiVersion, DriverVersion};
use crate::auxillary::enums::{
    AttachmentLoadOp, AttachmentStoreOp, AttributeLayout,
    BindPoint, BlendFactor, BlendOp,
    CompareOp, ComponentSwizzle, CullMode,
    DescriptorKind, DeviceKind, DrawMode,
    FrontFace,
    ImageFormat, ImageLayout,
    LogicOp,
    MemoryAllocatorKind,
    SharingMode, StencilOp,
    QueueKind,
    VertexInputRate, VertexTopology,
};
use crate::auxillary::flags::{
    AccessFlags,
    BufferUsageFlags,
    ColourComponentFlags,
    DependencyFlags, DeviceMemoryTypeFlags,
    HeapPropertyFlags,
    MemoryPropertyFlags,
    PipelineStage,
    SampleCount, SampleCountFlags, ShaderStage,
};
use crate::instance::Instance;


/***** GEOMETRY *****/
/// Defines a 2-dimensional offset with data type T.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Offset2D<T> {
    /// The X-coordinate of the offset.
    pub x : T,
    /// The Y-coordinate of the offset.
    pub y : T,
}

impl<T> Offset2D<T> {
    /// Constructor for the Offset2D.
    /// 
    /// # Generic arguments
    /// - `T`: The data type of the coordinates.
    /// 
    /// # Arguments
    /// - `x`: The X-coordinate of the offset.
    /// - `y`: The Y-coordinate of the offset.
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }



    /// Casts this Offset2D to another Offset2D with convertible types
    #[inline]
    pub fn cast<U: From<T>>(self) -> Offset2D<U> {
        Offset2D::new(U::from(self.x), U::from(self.y))
    }
}

impl<T> Display for Offset2D<T>
where
    T: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> From<vk::Offset2D> for Offset2D<T>
where
    T: From<i32>
{
    #[inline]
    fn from(value: vk::Offset2D) -> Self {
        Self {
            x : T::from(value.x),
            y : T::from(value.y),
        }
    }
}

impl<T> From<Offset2D<T>> for vk::Offset2D
where
    T: Into<i32>
{
    #[inline]
    fn from(value: Offset2D<T>) -> Self {
        Self {
            x : value.x.into(),
            y : value.y.into(),
        }
    }
}

impl<T> From<(T, T)> for Offset2D<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self {
            x : value.0,
            y : value.1,
        }
    }
}

impl<T> From<Offset2D<T>> for (T, T) {
    #[inline]
    fn from(value: Offset2D<T>) -> Self {
        (value.x, value.y)
    }
}

#[cfg(feature = "winit")]
impl<T> From<winit::dpi::PhysicalPosition<T>> for Offset2D<T> {
    #[inline]
    fn from(value: winit::dpi::PhysicalPosition<T>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(feature = "winit")]
impl<T> From<Offset2D<T>> for winit::dpi::PhysicalPosition<T> {
    #[inline]
    fn from(value: Offset2D<T>) -> Self {
        Self::new(value.x, value.y)
    }
}



/// Defines a 2-dimensional extent with data type T.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Extent2D<T> {
    /// The width of the extent.
    pub w : T,
    /// The height of the extent.
    pub h : T,
}

impl<T> Extent2D<T> {
    /// Constructor for the Extent2D.
    /// 
    /// # Generic arguments
    /// - `T`: The data type of the dimensions.
    /// 
    /// # Arguments
    /// - `w`: The width of the extent.
    /// - `h`: The height of the extent.
    #[inline]
    pub fn new(w: T, h: T) -> Self {
        Self { w, h }
    }



    /// Casts this Extent2D to another Extent2D with convertible types
    #[inline]
    pub fn cast<U: From<T>>(self) -> Extent2D<U> {
        Extent2D::new(U::from(self.w), U::from(self.h))
    }
}

impl<T> Display for Extent2D<T>
where
    T: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "({}, {})", self.w, self.h)
    }
}

impl<T> From<vk::Extent2D> for Extent2D<T>
where
    T: From<u32>
{
    #[inline]
    fn from(value: vk::Extent2D) -> Self {
        Self {
            w : T::from(value.width),
            h : T::from(value.height),
        }
    }
}

impl<T> From<Extent2D<T>> for vk::Extent2D
where
    T: Into<u32>
{
    #[inline]
    fn from(value: Extent2D<T>) -> Self {
        Self {
            width  : value.w.into(),
            height : value.h.into(),
        }
    }
}

impl<T> From<(T, T)> for Extent2D<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self {
            w : value.0,
            h : value.1,
        }
    }
}

impl<T> From<Extent2D<T>> for (T, T) {
    #[inline]
    fn from(value: Extent2D<T>) -> Self {
        (value.w, value.h)
    }
}

#[cfg(feature = "winit")]
impl<T> From<winit::dpi::PhysicalSize<T>> for Extent2D<T> {
    #[inline]
    fn from(value: winit::dpi::PhysicalSize<T>) -> Self {
        Self {
            w: value.width,
            h: value.height,
        }
    }
}

#[cfg(feature = "winit")]
impl<T> From<Extent2D<T>> for winit::dpi::PhysicalSize<T> {
    #[inline]
    fn from(value: Extent2D<T>) -> Self {
        Self::new(value.w, value.h)
    }
}



/// Defines a 2-dimensional rectangle with an offset (of datatype T) and an extent (of datatype U).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rect2D<T, U = T> {
    /// The offset of the top-left corner of the rectangle.
    pub offset : Offset2D<T>,
    /// The extent of rectangle.
    pub extent : Extent2D<U>,
}

impl<T, U> Rect2D<T, U> {
    /// Constructor for the Rect2D.
    /// 
    /// # Generic arguments
    /// - `T`: The data type of the offset.
    /// - `U`: The data type of the extent.
    /// 
    /// # Arguments
    /// - `x`: The X-coordinate of the offset.
    /// - `y`: The Y-coordinate of the offset.
    /// - `w`: The width of the extent.
    /// - `h`: The height of the extent.
    #[inline]
    pub fn new(x: T, y: T, w: U, h: U) -> Self {
        Self {
            offset : Offset2D::new(x, y),
            extent : Extent2D::new(w, h),
        }
    }

    /// Constructor for the Rect2D that takes a separate offset and extend.
    /// 
    /// # Generic arguments
    /// - `T`: The data type of the offset.
    /// - `U`: The data type of the extent.
    /// 
    /// # Arguments
    /// - `offset`: The offset of the rectangle.
    /// - `extent`: The extent of the rectangle.
    #[inline]
    pub fn from_raw(offset: Offset2D<T>, extent: Extent2D<U>) -> Self {
        Self {
            offset,
            extent,
        }
    }



    /// Casts this Rect2D to another Rect2D with convertible types
    #[inline]
    pub fn cast<V: From<T>, W: From<U>>(self) -> Rect2D<V, W> {
        Rect2D::from_raw(self.offset.cast(), self.extent.cast())
    }



    /// Returns the X-coordinate of the rectangle's offset.
    #[inline]
    pub fn x(&self) -> T where T: Copy { self.offset.x }

    /// Returns the Y-coordinate of the rectangle's offset.
    #[inline]
    pub fn y(&self) -> T where T: Copy { self.offset.y }

    /// Returns the width of the rectangle's extent.
    #[inline]
    pub fn w(&self) -> U where U: Copy { self.extent.w }

    /// Returns the height of the rectangle's extent.
    #[inline]
    pub fn h(&self) -> U where U: Copy { self.extent.h }
}

impl<T, U> From<vk::Rect2D> for Rect2D<T, U>
where
    T: From<i32>,
    U: From<u32>,
{
    #[inline]
    fn from(value: vk::Rect2D) -> Self {
        Self {
            offset : value.offset.into(),
            extent : value.extent.into(),
        }
    }
}

impl<T, U> From<Rect2D<T, U>> for vk::Rect2D
where
    T: Into<i32>,
    U: Into<u32>,
{
    #[inline]
    fn from(value: Rect2D<T, U>) -> Self {
        Self {
            offset : value.offset.into(),
            extent : value.extent.into(),
        }
    }
}





/***** PHYSICAL DEVICES *****/
/// Contains the properties of a physical device.
#[derive(Clone, Debug)]
pub struct PhysicalDeviceProperties {
    /// The name of the device.
    pub name : String,
    /// The type of the device.
    pub kind : DeviceKind,

    /// The API version maximally supported by the Device.
    pub api_version    : ApiVersion,
    /// The driver version maximally supported by the Device. The actual encoding is vendor-specific.
    pub driver_version : DriverVersion,
    /// The vendor identifier.
    pub vendor_id      : u32,
    /// The device identifier (vendor-unique, I assume).
    pub device_id      : u32,
    /// The UUID of this device w.r.t. pipeline caches.
    pub pipeline_cache_uuid : [u8; vk::UUID_SIZE],

    /// A struct describing all sorts of limits for this Device.
    pub limits : PhysicalDeviceLimits,
    /// A struct describing the sparse matrix properties supported by this device.
    pub sparse : PhysicalDeviceSparseProperties,
}

impl From<vk::PhysicalDeviceProperties> for PhysicalDeviceProperties {
    fn from(value: vk::PhysicalDeviceProperties) -> Self {
        // Convert the name into a normal string
        let name: &str = unsafe{ CStr::from_ptr(value.device_name.as_ptr()) }.to_str().unwrap_or_else(|err| panic!("Failed to convert device name to String: {}", err));

        // Return the proper struct
        Self {
            name : name.into(),
            kind : value.device_type.into(),

            api_version         : value.api_version.into(),
            driver_version      : value.driver_version.into(),
            vendor_id           : value.vendor_id,
            device_id           : value.device_id,
            pipeline_cache_uuid : value.pipeline_cache_uuid,

            limits : value.limits.into(),
            sparse : value.sparse_properties.into(),
        }
    }
}

impl From<PhysicalDeviceProperties> for vk::PhysicalDeviceProperties {
    fn from(value: PhysicalDeviceProperties) -> Self {
        // Convert the name to a C-string
        let name_len = value.name.len();
        if name_len > vk::MAX_PHYSICAL_DEVICE_NAME_SIZE { panic!("Device name '{}' is too long", value.name); }
        let cname : CString = to_cstring!(value.name);
        let mut bname : [ i8; vk::MAX_PHYSICAL_DEVICE_NAME_SIZE ] = [ 0; vk::MAX_PHYSICAL_DEVICE_NAME_SIZE ];
        bname.copy_from_slice(unsafe{ slice::from_raw_parts(CString::as_bytes_with_nul(&cname).as_ptr() as *const i8, name_len) });

        // Return a new Vulkan counterpart
        Self {
            device_name : bname,
            device_type : value.kind.into(),

            api_version         : value.api_version.into(),
            driver_version      : value.driver_version.into(),
            vendor_id           : value.vendor_id,
            device_id           : value.device_id,
            pipeline_cache_uuid : value.pipeline_cache_uuid,

            limits            : value.limits.into(),
            sparse_properties : value.sparse.into(),
        }
    }
}



/// Contains the limits of a specified PhysicalDevice.
#[derive(Clone, Debug)]
pub struct PhysicalDeviceLimits {
    pub max_image_dimension_1d : u32,
    pub max_image_dimension_2d : u32,
    pub max_image_dimension_3d : u32,
    pub max_image_dimension_cube : u32,
    pub max_image_array_layers : u32,
    pub max_texel_buffer_elements : u32,
    pub max_uniform_buffer_range : u32,
    pub max_storage_buffer_range : u32,
    pub max_push_constants_size : u32,
    pub max_memory_allocation_count : u32,
    pub max_sampler_allocation_count : u32,
    pub buffer_image_granularity : vk::DeviceSize,
    pub sparse_address_space_size : vk::DeviceSize,
    pub max_bound_descriptor_sets : u32,
    pub max_per_stage_descriptor_samplers : u32,
    pub max_per_stage_descriptor_uniform_buffers : u32,
    pub max_per_stage_descriptor_storage_buffers : u32,
    pub max_per_stage_descriptor_sampled_images : u32,
    pub max_per_stage_descriptor_storage_images : u32,
    pub max_per_stage_descriptor_input_attachments : u32,
    pub max_per_stage_resources : u32,
    pub max_descriptor_set_samplers : u32,
    pub max_descriptor_set_uniform_buffers : u32,
    pub max_descriptor_set_uniform_buffers_dynamic : u32,
    pub max_descriptor_set_storage_buffers : u32,
    pub max_descriptor_set_storage_buffers_dynamic : u32,
    pub max_descriptor_set_sampled_images : u32,
    pub max_descriptor_set_storage_images : u32,
    pub max_descriptor_set_input_attachments : u32,
    pub max_vertex_input_attributes : u32,
    pub max_vertex_input_bindings : u32,
    pub max_vertex_input_attribute_offset : u32,
    pub max_vertex_input_binding_stride : u32,
    pub max_vertex_output_components : u32,
    pub max_tessellation_generation_level : u32,
    pub max_tessellation_patch_size : u32,
    pub max_tessellation_control_per_vertex_input_components : u32,
    pub max_tessellation_control_per_vertex_output_components : u32,
    pub max_tessellation_control_per_patch_output_components : u32,
    pub max_tessellation_control_total_output_components : u32,
    pub max_tessellation_evaluation_input_components : u32,
    pub max_tessellation_evaluation_output_components : u32,
    pub max_geometry_shader_invocations : u32,
    pub max_geometry_input_components : u32,
    pub max_geometry_output_components : u32,
    pub max_geometry_output_vertices : u32,
    pub max_geometry_total_output_components : u32,
    pub max_fragment_input_components : u32,
    pub max_fragment_output_attachments : u32,
    pub max_fragment_dual_src_attachments : u32,
    pub max_fragment_combined_output_resources : u32,
    pub max_compute_shared_memory_size : u32,
    pub max_compute_work_group_count : [ u32; 3 ],
    pub max_compute_work_group_invocations : u32,
    pub max_compute_work_group_size : [ u32; 3 ],
    pub sub_pixel_precision_bits : u32,
    pub sub_texel_precision_bits : u32,
    pub mipmap_precision_bits : u32,
    pub max_draw_indexed_index_value : u32,
    pub max_draw_indirect_count : u32,
    pub max_sampler_lod_bias : f32,
    pub max_sampler_anisotropy : f32,
    pub max_viewports : u32,
    pub max_viewport_dimensions : [ u32; 2 ],
    pub viewport_bounds_range : [ f32; 2 ],
    pub viewport_sub_pixel_bits : u32,
    pub min_memory_map_alignment : usize,
    pub min_texel_buffer_offset_alignment : vk::DeviceSize,
    pub min_uniform_buffer_offset_alignment : vk::DeviceSize,
    pub min_storage_buffer_offset_alignment : vk::DeviceSize,
    pub min_texel_offset : i32,
    pub max_texel_offset : u32,
    pub min_texel_gather_offset : i32,
    pub max_texel_gather_offset : u32,
    pub min_interpolation_offset : f32,
    pub max_interpolation_offset : f32,
    pub sub_pixel_interpolation_offset_bits : u32,
    pub max_framebuffer_width : u32,
    pub max_framebuffer_height : u32,
    pub max_framebuffer_layers : u32,
    pub framebuffer_color_sample_counts : SampleCountFlags,
    pub framebuffer_depth_sample_counts : SampleCountFlags,
    pub framebuffer_stencil_sample_counts : SampleCountFlags,
    pub framebuffer_no_attachments_sample_counts : SampleCountFlags,
    pub max_color_attachments : u32,
    pub sampled_image_color_sample_counts : SampleCountFlags,
    pub sampled_image_integer_sample_counts : SampleCountFlags,
    pub sampled_image_depth_sample_counts : SampleCountFlags,
    pub sampled_image_stencil_sample_counts : SampleCountFlags,
    pub storage_image_sample_counts : SampleCountFlags,
    pub max_sample_mask_words : u32,
    pub timestamp_compute_and_graphics : bool,
    pub timestamp_period : f32,
    pub max_clip_distances : u32,
    pub max_cull_distances : u32,
    pub max_combined_clip_and_cull_distances : u32,
    pub discrete_queue_priorities : u32,
    pub point_size_range : [ f32; 2 ],
    pub line_width_range : [ f32; 2 ],
    pub point_size_granularity : f32,
    pub line_width_granularity : f32,
    pub strict_lines : bool,
    pub standard_sample_locations : bool,
    pub optimal_buffer_copy_offset_alignment : vk::DeviceSize,
    pub optimal_buffer_copy_row_pitch_alignment : vk::DeviceSize,
    pub non_coherent_atom_size : vk::DeviceSize,
}

impl From<vk::PhysicalDeviceLimits> for PhysicalDeviceLimits {
    fn from(value: vk::PhysicalDeviceLimits) -> Self {
        Self {
            max_image_dimension_1d                                : value.max_image_dimension1_d,
            max_image_dimension_2d                                : value.max_image_dimension2_d,
            max_image_dimension_3d                                : value.max_image_dimension3_d,
            max_image_dimension_cube                              : value.max_image_dimension_cube,
            max_image_array_layers                                : value.max_image_array_layers,
            max_texel_buffer_elements                             : value.max_texel_buffer_elements,
            max_uniform_buffer_range                              : value.max_uniform_buffer_range,
            max_storage_buffer_range                              : value.max_storage_buffer_range,
            max_push_constants_size                               : value.max_push_constants_size,
            max_memory_allocation_count                           : value.max_memory_allocation_count,
            max_sampler_allocation_count                          : value.max_sampler_allocation_count,
            buffer_image_granularity                              : value.buffer_image_granularity,
            sparse_address_space_size                             : value.sparse_address_space_size,
            max_bound_descriptor_sets                             : value.max_bound_descriptor_sets,
            max_per_stage_descriptor_samplers                     : value.max_per_stage_descriptor_samplers,
            max_per_stage_descriptor_uniform_buffers              : value.max_per_stage_descriptor_uniform_buffers,
            max_per_stage_descriptor_storage_buffers              : value.max_per_stage_descriptor_storage_buffers,
            max_per_stage_descriptor_sampled_images               : value.max_per_stage_descriptor_sampled_images,
            max_per_stage_descriptor_storage_images               : value.max_per_stage_descriptor_storage_images,
            max_per_stage_descriptor_input_attachments            : value.max_per_stage_descriptor_input_attachments,
            max_per_stage_resources                               : value.max_per_stage_resources,
            max_descriptor_set_samplers                           : value.max_descriptor_set_samplers,
            max_descriptor_set_uniform_buffers                    : value.max_descriptor_set_uniform_buffers,
            max_descriptor_set_uniform_buffers_dynamic            : value.max_descriptor_set_uniform_buffers_dynamic,
            max_descriptor_set_storage_buffers                    : value.max_descriptor_set_storage_buffers,
            max_descriptor_set_storage_buffers_dynamic            : value.max_descriptor_set_storage_buffers_dynamic,
            max_descriptor_set_sampled_images                     : value.max_descriptor_set_sampled_images,
            max_descriptor_set_storage_images                     : value.max_descriptor_set_storage_images,
            max_descriptor_set_input_attachments                  : value.max_descriptor_set_input_attachments,
            max_vertex_input_attributes                           : value.max_vertex_input_attributes,
            max_vertex_input_bindings                             : value.max_vertex_input_bindings,
            max_vertex_input_attribute_offset                     : value.max_vertex_input_attribute_offset,
            max_vertex_input_binding_stride                       : value.max_vertex_input_binding_stride,
            max_vertex_output_components                          : value.max_vertex_output_components,
            max_tessellation_generation_level                     : value.max_tessellation_generation_level,
            max_tessellation_patch_size                           : value.max_tessellation_patch_size,
            max_tessellation_control_per_vertex_input_components  : value.max_tessellation_control_per_vertex_input_components,
            max_tessellation_control_per_vertex_output_components : value.max_tessellation_control_per_vertex_output_components,
            max_tessellation_control_per_patch_output_components  : value.max_tessellation_control_per_patch_output_components,
            max_tessellation_control_total_output_components      : value.max_tessellation_control_total_output_components,
            max_tessellation_evaluation_input_components          : value.max_tessellation_evaluation_input_components,
            max_tessellation_evaluation_output_components         : value.max_tessellation_evaluation_output_components,
            max_geometry_shader_invocations                       : value.max_geometry_shader_invocations,
            max_geometry_input_components                         : value.max_geometry_input_components,
            max_geometry_output_components                        : value.max_geometry_output_components,
            max_geometry_output_vertices                          : value.max_geometry_output_vertices,
            max_geometry_total_output_components                  : value.max_geometry_total_output_components,
            max_fragment_input_components                         : value.max_fragment_input_components,
            max_fragment_output_attachments                       : value.max_fragment_output_attachments,
            max_fragment_dual_src_attachments                     : value.max_fragment_dual_src_attachments,
            max_fragment_combined_output_resources                : value.max_fragment_combined_output_resources,
            max_compute_shared_memory_size                        : value.max_compute_shared_memory_size,
            max_compute_work_group_count                          : value.max_compute_work_group_count,
            max_compute_work_group_invocations                    : value.max_compute_work_group_invocations,
            max_compute_work_group_size                           : value.max_compute_work_group_size,
            sub_pixel_precision_bits                              : value.sub_pixel_precision_bits,
            sub_texel_precision_bits                              : value.sub_texel_precision_bits,
            mipmap_precision_bits                                 : value.mipmap_precision_bits,
            max_draw_indexed_index_value                          : value.max_draw_indexed_index_value,
            max_draw_indirect_count                               : value.max_draw_indirect_count,
            max_sampler_lod_bias                                  : value.max_sampler_lod_bias,
            max_sampler_anisotropy                                : value.max_sampler_anisotropy,
            max_viewports                                         : value.max_viewports,
            max_viewport_dimensions                               : value.max_viewport_dimensions,
            viewport_bounds_range                                 : value.viewport_bounds_range,
            viewport_sub_pixel_bits                               : value.viewport_sub_pixel_bits,
            min_memory_map_alignment                              : value.min_memory_map_alignment,
            min_texel_buffer_offset_alignment                     : value.min_texel_buffer_offset_alignment,
            min_uniform_buffer_offset_alignment                   : value.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment                   : value.min_storage_buffer_offset_alignment,
            min_texel_offset                                      : value.min_texel_offset,
            max_texel_offset                                      : value.max_texel_offset,
            min_texel_gather_offset                               : value.min_texel_gather_offset,
            max_texel_gather_offset                               : value.max_texel_gather_offset,
            min_interpolation_offset                              : value.min_interpolation_offset,
            max_interpolation_offset                              : value.max_interpolation_offset,
            sub_pixel_interpolation_offset_bits                   : value.sub_pixel_interpolation_offset_bits,
            max_framebuffer_width                                 : value.max_framebuffer_width,
            max_framebuffer_height                                : value.max_framebuffer_height,
            max_framebuffer_layers                                : value.max_framebuffer_layers,
            framebuffer_color_sample_counts                       : value.framebuffer_color_sample_counts.into(),
            framebuffer_depth_sample_counts                       : value.framebuffer_depth_sample_counts.into(),
            framebuffer_stencil_sample_counts                     : value.framebuffer_stencil_sample_counts.into(),
            framebuffer_no_attachments_sample_counts              : value.framebuffer_no_attachments_sample_counts.into(),
            max_color_attachments                                 : value.max_color_attachments,
            sampled_image_color_sample_counts                     : value.sampled_image_color_sample_counts.into(),
            sampled_image_integer_sample_counts                   : value.sampled_image_integer_sample_counts.into(),
            sampled_image_depth_sample_counts                     : value.sampled_image_depth_sample_counts.into(),
            sampled_image_stencil_sample_counts                   : value.sampled_image_stencil_sample_counts.into(),
            storage_image_sample_counts                           : value.storage_image_sample_counts.into(),
            max_sample_mask_words                                 : value.max_sample_mask_words,
            timestamp_compute_and_graphics                        : if value.timestamp_compute_and_graphics == vk::TRUE { true } else { false },
            timestamp_period                                      : value.timestamp_period,
            max_clip_distances                                    : value.max_clip_distances,
            max_cull_distances                                    : value.max_cull_distances,
            max_combined_clip_and_cull_distances                  : value.max_combined_clip_and_cull_distances,
            discrete_queue_priorities                             : value.discrete_queue_priorities,
            point_size_range                                      : value.point_size_range,
            line_width_range                                      : value.line_width_range,
            point_size_granularity                                : value.point_size_granularity,
            line_width_granularity                                : value.line_width_granularity,
            strict_lines                                          : if value.strict_lines == vk::TRUE { true } else { false },
            standard_sample_locations                             : if value.standard_sample_locations == vk::TRUE { true } else { false },
            optimal_buffer_copy_offset_alignment                  : value.optimal_buffer_copy_offset_alignment,
            optimal_buffer_copy_row_pitch_alignment               : value.optimal_buffer_copy_row_pitch_alignment,
            non_coherent_atom_size                                : value.non_coherent_atom_size,            
        }
    }
}

impl From<PhysicalDeviceLimits> for vk::PhysicalDeviceLimits {
    #[inline]
    fn from(value: PhysicalDeviceLimits) -> Self {
        Self {
            max_image_dimension1_d                                : value.max_image_dimension_1d,
            max_image_dimension2_d                                : value.max_image_dimension_2d,
            max_image_dimension3_d                                : value.max_image_dimension_3d,
            max_image_dimension_cube                              : value.max_image_dimension_cube,
            max_image_array_layers                                : value.max_image_array_layers,
            max_texel_buffer_elements                             : value.max_texel_buffer_elements,
            max_uniform_buffer_range                              : value.max_uniform_buffer_range,
            max_storage_buffer_range                              : value.max_storage_buffer_range,
            max_push_constants_size                               : value.max_push_constants_size,
            max_memory_allocation_count                           : value.max_memory_allocation_count,
            max_sampler_allocation_count                          : value.max_sampler_allocation_count,
            buffer_image_granularity                              : value.buffer_image_granularity,
            sparse_address_space_size                             : value.sparse_address_space_size,
            max_bound_descriptor_sets                             : value.max_bound_descriptor_sets,
            max_per_stage_descriptor_samplers                     : value.max_per_stage_descriptor_samplers,
            max_per_stage_descriptor_uniform_buffers              : value.max_per_stage_descriptor_uniform_buffers,
            max_per_stage_descriptor_storage_buffers              : value.max_per_stage_descriptor_storage_buffers,
            max_per_stage_descriptor_sampled_images               : value.max_per_stage_descriptor_sampled_images,
            max_per_stage_descriptor_storage_images               : value.max_per_stage_descriptor_storage_images,
            max_per_stage_descriptor_input_attachments            : value.max_per_stage_descriptor_input_attachments,
            max_per_stage_resources                               : value.max_per_stage_resources,
            max_descriptor_set_samplers                           : value.max_descriptor_set_samplers,
            max_descriptor_set_uniform_buffers                    : value.max_descriptor_set_uniform_buffers,
            max_descriptor_set_uniform_buffers_dynamic            : value.max_descriptor_set_uniform_buffers_dynamic,
            max_descriptor_set_storage_buffers                    : value.max_descriptor_set_storage_buffers,
            max_descriptor_set_storage_buffers_dynamic            : value.max_descriptor_set_storage_buffers_dynamic,
            max_descriptor_set_sampled_images                     : value.max_descriptor_set_sampled_images,
            max_descriptor_set_storage_images                     : value.max_descriptor_set_storage_images,
            max_descriptor_set_input_attachments                  : value.max_descriptor_set_input_attachments,
            max_vertex_input_attributes                           : value.max_vertex_input_attributes,
            max_vertex_input_bindings                             : value.max_vertex_input_bindings,
            max_vertex_input_attribute_offset                     : value.max_vertex_input_attribute_offset,
            max_vertex_input_binding_stride                       : value.max_vertex_input_binding_stride,
            max_vertex_output_components                          : value.max_vertex_output_components,
            max_tessellation_generation_level                     : value.max_tessellation_generation_level,
            max_tessellation_patch_size                           : value.max_tessellation_patch_size,
            max_tessellation_control_per_vertex_input_components  : value.max_tessellation_control_per_vertex_input_components,
            max_tessellation_control_per_vertex_output_components : value.max_tessellation_control_per_vertex_output_components,
            max_tessellation_control_per_patch_output_components  : value.max_tessellation_control_per_patch_output_components,
            max_tessellation_control_total_output_components      : value.max_tessellation_control_total_output_components,
            max_tessellation_evaluation_input_components          : value.max_tessellation_evaluation_input_components,
            max_tessellation_evaluation_output_components         : value.max_tessellation_evaluation_output_components,
            max_geometry_shader_invocations                       : value.max_geometry_shader_invocations,
            max_geometry_input_components                         : value.max_geometry_input_components,
            max_geometry_output_components                        : value.max_geometry_output_components,
            max_geometry_output_vertices                          : value.max_geometry_output_vertices,
            max_geometry_total_output_components                  : value.max_geometry_total_output_components,
            max_fragment_input_components                         : value.max_fragment_input_components,
            max_fragment_output_attachments                       : value.max_fragment_output_attachments,
            max_fragment_dual_src_attachments                     : value.max_fragment_dual_src_attachments,
            max_fragment_combined_output_resources                : value.max_fragment_combined_output_resources,
            max_compute_shared_memory_size                        : value.max_compute_shared_memory_size,
            max_compute_work_group_count                          : value.max_compute_work_group_count,
            max_compute_work_group_invocations                    : value.max_compute_work_group_invocations,
            max_compute_work_group_size                           : value.max_compute_work_group_size,
            sub_pixel_precision_bits                              : value.sub_pixel_precision_bits,
            sub_texel_precision_bits                              : value.sub_texel_precision_bits,
            mipmap_precision_bits                                 : value.mipmap_precision_bits,
            max_draw_indexed_index_value                          : value.max_draw_indexed_index_value,
            max_draw_indirect_count                               : value.max_draw_indirect_count,
            max_sampler_lod_bias                                  : value.max_sampler_lod_bias,
            max_sampler_anisotropy                                : value.max_sampler_anisotropy,
            max_viewports                                         : value.max_viewports,
            max_viewport_dimensions                               : value.max_viewport_dimensions,
            viewport_bounds_range                                 : value.viewport_bounds_range,
            viewport_sub_pixel_bits                               : value.viewport_sub_pixel_bits,
            min_memory_map_alignment                              : value.min_memory_map_alignment,
            min_texel_buffer_offset_alignment                     : value.min_texel_buffer_offset_alignment,
            min_uniform_buffer_offset_alignment                   : value.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment                   : value.min_storage_buffer_offset_alignment,
            min_texel_offset                                      : value.min_texel_offset,
            max_texel_offset                                      : value.max_texel_offset,
            min_texel_gather_offset                               : value.min_texel_gather_offset,
            max_texel_gather_offset                               : value.max_texel_gather_offset,
            min_interpolation_offset                              : value.min_interpolation_offset,
            max_interpolation_offset                              : value.max_interpolation_offset,
            sub_pixel_interpolation_offset_bits                   : value.sub_pixel_interpolation_offset_bits,
            max_framebuffer_width                                 : value.max_framebuffer_width,
            max_framebuffer_height                                : value.max_framebuffer_height,
            max_framebuffer_layers                                : value.max_framebuffer_layers,
            framebuffer_color_sample_counts                       : value.framebuffer_color_sample_counts.into(),
            framebuffer_depth_sample_counts                       : value.framebuffer_depth_sample_counts.into(),
            framebuffer_stencil_sample_counts                     : value.framebuffer_stencil_sample_counts.into(),
            framebuffer_no_attachments_sample_counts              : value.framebuffer_no_attachments_sample_counts.into(),
            max_color_attachments                                 : value.max_color_attachments,
            sampled_image_color_sample_counts                     : value.sampled_image_color_sample_counts.into(),
            sampled_image_integer_sample_counts                   : value.sampled_image_integer_sample_counts.into(),
            sampled_image_depth_sample_counts                     : value.sampled_image_depth_sample_counts.into(),
            sampled_image_stencil_sample_counts                   : value.sampled_image_stencil_sample_counts.into(),
            storage_image_sample_counts                           : value.storage_image_sample_counts.into(),
            max_sample_mask_words                                 : value.max_sample_mask_words,
            timestamp_compute_and_graphics                        : if value.timestamp_compute_and_graphics { vk::TRUE } else { vk::FALSE },
            timestamp_period                                      : value.timestamp_period,
            max_clip_distances                                    : value.max_clip_distances,
            max_cull_distances                                    : value.max_cull_distances,
            max_combined_clip_and_cull_distances                  : value.max_combined_clip_and_cull_distances,
            discrete_queue_priorities                             : value.discrete_queue_priorities,
            point_size_range                                      : value.point_size_range,
            line_width_range                                      : value.line_width_range,
            point_size_granularity                                : value.point_size_granularity,
            line_width_granularity                                : value.line_width_granularity,
            strict_lines                                          : if value.strict_lines { vk::TRUE } else { vk::FALSE },
            standard_sample_locations                             : if value.standard_sample_locations { vk::TRUE } else { vk::FALSE },
            optimal_buffer_copy_offset_alignment                  : value.optimal_buffer_copy_offset_alignment,
            optimal_buffer_copy_row_pitch_alignment               : value.optimal_buffer_copy_row_pitch_alignment,
            non_coherent_atom_size                                : value.non_coherent_atom_size,
        }
    }
}



/// A struct describing the sparse matrix properties supported by a PhysicalDevice.
#[derive(Clone, Debug)]
pub struct PhysicalDeviceSparseProperties {
    /// Indicates whether the device uses the standard-defined image block shapes for all single-sample, 2D sparse resources.
    pub standard_2d_block_shape             : bool,
    /// Indicates whether the device uses the standard-defined image block shapes for all multi-sample, 2D sparse resources.
    pub standard_2d_multisample_block_shape : bool,
    /// Indicates whether the device uses the standard-defined image block shapes for all single-sample, 3D sparse resources.
    pub standard_3d_block_shape             : bool,
    /// Indicates whether the device may place mip level dimensions that are not integer multiples of the corresponding dimensions of the sparse block image in the mip tail.
    pub aligned_mip_size                    : bool,
    /// Indicates whether the device can consistently access non-resident regions of a resource. Any such regions will be treated as-if they always contain 0.
    pub non_resident_strict                 : bool,
}

impl From<vk::PhysicalDeviceSparseProperties> for PhysicalDeviceSparseProperties {
    #[inline]
    fn from(value: vk::PhysicalDeviceSparseProperties) -> Self {
        Self {
            standard_2d_block_shape             : value.residency_standard2_d_block_shape == vk::TRUE,
            standard_2d_multisample_block_shape : value.residency_standard2_d_multisample_block_shape == vk::TRUE,
            standard_3d_block_shape             : value.residency_standard3_d_block_shape == vk::TRUE,
            aligned_mip_size                    : value.residency_aligned_mip_size == vk::TRUE,
            non_resident_strict                 : value.residency_non_resident_strict == vk::TRUE,
        }
    }
}

impl From<PhysicalDeviceSparseProperties> for vk::PhysicalDeviceSparseProperties {
    #[inline]
    fn from(value: PhysicalDeviceSparseProperties) -> Self {
        Self {
            residency_standard2_d_block_shape             : if value.standard_2d_block_shape { vk::TRUE } else { vk::FALSE },
            residency_standard2_d_multisample_block_shape : if value.standard_2d_multisample_block_shape { vk::TRUE } else { vk::FALSE },
            residency_standard3_d_block_shape             : if value.standard_3d_block_shape { vk::TRUE } else { vk::FALSE },
            residency_aligned_mip_size                    : if value.aligned_mip_size { vk::TRUE } else { vk::FALSE },
            residency_non_resident_strict                 : if value.non_resident_strict { vk::TRUE } else { vk::FALSE },
        }
    }
}





/***** DEVICES *****/
/// Lists information about a GPU (for use when listing them).
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    /// The index of the Device.
    pub index : usize,
    /// The name of the Device.
    pub name  : String,
    /// The kind of the Device.
    pub kind  : DeviceKind,

    /// The memory properties of the Device.
    pub mem_props : DeviceMemoryProperties,
}



/// Lists information about a monitor (for use when listing them).
#[derive(Clone, Debug)]
pub struct MonitorInfo {
    /// The index of the monitor.
    pub index       : usize,
    /// The name of the monitor.
    pub name        : String,
    /// The resolution of the monitor.
    pub resolution  : (u32, u32),
    /// The supported video modes of this monitor.
    pub video_modes : Vec<MonitorVideoMode>,
}



/// Contains the information of a single video mode in the MonitorInfo.
#[derive(Clone, Debug)]
pub struct MonitorVideoMode {
    /// The resolution for this video mode.
    pub resolution   : (u32, u32),
    /// The refresh rate (in Hz) for this video mode.
    pub refresh_rate : u16,
    /// The bit depth (in bits-per-pixel) for this video mode.
    pub bit_depth    : u16,
}

impl Display for MonitorVideoMode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}x{}@{} ({} bpp)", self.resolution.0, self.resolution.1, self.refresh_rate, self.bit_depth)
    }
}

#[cfg(feature = "winit")]
impl From<winit::monitor::VideoMode> for MonitorVideoMode {
    #[inline]
    fn from(value: winit::monitor::VideoMode) -> Self {
        Self {
            resolution   : value.size().into(),
            refresh_rate : value.refresh_rate(),
            bit_depth    : value.bit_depth(),
        }
    }
}



/// Lists information about a Device's memory.
#[derive(Clone, Debug)]
pub struct DeviceMemoryProperties {
    /// The list of heaps supported by this device.
    pub heaps : Vec<DeviceMemoryHeapInfo>,
    /// The types of memory supported by this device.
    pub types : Vec<DeviceMemoryTypeInfo>,
}

impl From<vk::PhysicalDeviceMemoryProperties> for DeviceMemoryProperties {
    #[inline]
    fn from(value: vk::PhysicalDeviceMemoryProperties) -> Self {
        Self {
            heaps : unsafe { slice::from_raw_parts::<vk::MemoryHeap>(value.memory_heaps.as_ptr(), value.memory_heap_count as usize) }.iter().map(|info| info.into()).collect(),
            types : unsafe { slice::from_raw_parts::<vk::MemoryType>(value.memory_types.as_ptr(), value.memory_type_count as usize) }.iter().map(|info| info.into()).collect(),
        }
    }
}

impl From<DeviceMemoryProperties> for vk::PhysicalDeviceMemoryProperties {
    fn from(value: DeviceMemoryProperties) -> Self {
        // Prepare the fixed-size memory arrays
        let mut memory_heaps: [vk::MemoryHeap; 16] = Default::default();
        let mut memory_types: [vk::MemoryType; 32] = Default::default();

        // Copy the infos over to it
        let memory_heap_count: u32 = value.heaps.len() as u32;
        let memory_type_count: u32 = value.types.len() as u32;
        for (i, info) in value.heaps.into_iter().enumerate() { memory_heaps[i] = info.into(); }
        for (i, info) in value.types.into_iter().enumerate() { memory_types[i] = info.into(); }

        // Wrap them in a return struct
        Self {
            memory_heap_count,
            memory_heaps,
            memory_type_count,
            memory_types,
        }
    }
}



/// Lists information about each heap on the Device.
#[derive(Clone, Debug)]
pub struct DeviceMemoryHeapInfo {
    /// The size of this memory heap.
    pub size  : usize,
    /// Lists properties about the memory heap.
    pub props : HeapPropertyFlags,
}

impl From<vk::MemoryHeap> for DeviceMemoryHeapInfo {
    #[inline]
    fn from(value: vk::MemoryHeap) -> Self {
        // Use the referenced version
        Self::from(&value)
    }
}

impl From<&vk::MemoryHeap> for DeviceMemoryHeapInfo {
    #[inline]
    fn from(value: &vk::MemoryHeap) -> Self {
        Self {
            size  : value.size as usize,
            props : value.flags.into(),
        }
    }
}

impl From<DeviceMemoryHeapInfo> for vk::MemoryHeap {
    #[inline]
    fn from(value: DeviceMemoryHeapInfo) -> Self {
        // Use the referenced version
        Self::from(&value)
    }
}

impl From<&DeviceMemoryHeapInfo> for vk::MemoryHeap {
    #[inline]
    fn from(value: &DeviceMemoryHeapInfo) -> Self {
        Self {
            size  : value.size as vk::DeviceSize,
            flags : value.props.into(),
        }
    }
}



/// Lists information about each type of memory on the Device.
#[derive(Clone, Debug)]
pub struct DeviceMemoryTypeInfo {
    /// The index of the corresponding heap.
    pub heap_index : u32,
    /// The property flags supported by this type.
    pub props      : MemoryPropertyFlags,
}

impl From<vk::MemoryType> for DeviceMemoryTypeInfo {
    #[inline]
    fn from(value: vk::MemoryType) -> Self {
        // Use the referenced version
        Self::from(&value)
    }
}

impl From<&vk::MemoryType> for DeviceMemoryTypeInfo {
    #[inline]
    fn from(value: &vk::MemoryType) -> Self {
        Self {
            heap_index : value.heap_index,
            props      : value.property_flags.into(),
        }
    }
}

impl From<DeviceMemoryTypeInfo> for vk::MemoryType {
    #[inline]
    fn from(value: DeviceMemoryTypeInfo) -> Self {
        // Use the referenced version
        Self::from(&value)
    }
}

impl From<&DeviceMemoryTypeInfo> for vk::MemoryType {
    #[inline]
    fn from(value: &DeviceMemoryTypeInfo) -> Self {
        Self {
            heap_index     : value.heap_index,
            property_flags : value.props.into(),
        }
    }
}



/// The features that we can enable on a Device.
#[derive(Clone, Debug)]
pub struct DeviceFeatures {
    
}

impl DeviceFeatures {
    /// Constant default() function.
    #[inline]
    pub const fn cdefault() -> Self {
        Self {}
    }
}

impl Default for DeviceFeatures {
    #[inline]
    fn default() -> Self { Self::cdefault() }
}

impl From<vk::PhysicalDeviceFeatures> for DeviceFeatures {
    #[inline]
    fn from(value: vk::PhysicalDeviceFeatures) -> Self {
        // Use the reference one
        Self::from(&value)
    }
}

impl From<&vk::PhysicalDeviceFeatures> for DeviceFeatures {
    #[inline]
    fn from(_value: &vk::PhysicalDeviceFeatures) -> Self {
        Self {
            
        }
    }
}

impl From<DeviceFeatures> for vk::PhysicalDeviceFeatures {
    #[inline]
    fn from(value: DeviceFeatures) -> Self {
        // Use the reference one
        Self::from(&value)
    }
}

impl From<&DeviceFeatures> for vk::PhysicalDeviceFeatures {
    #[inline]
    fn from(_value: &DeviceFeatures) -> Self {
        Self {
            // Set the rest to off
            ..Default::default()
        }
    }
}





/***** QUEUES *****/
/// Contains information about the queue families for an instantiated GPU.
#[derive(Clone, Debug)]
pub struct QueueFamilyInfo {
    /// The index of the queue we're going to use for graphics operations.
    pub graphics : u32,
    /// The index of the queue we're going to use for memory operations.
    pub memory   : u32,
    /// The index of the queue we're going to use for present operations. Always the same as `graphics`.
    pub present  : u32,
    /// The index of the queue we're going to use for compute operations.
    pub compute  : u32,
}

impl QueueFamilyInfo {
    /// Constructor for the QueueFamilyInfo.
    /// 
    /// Maps the queue families of the given PhysicalDevice to their usage. Will try to use as many different queue families as possible.
    /// 
    /// # Arguments
    /// - `instance`: A reference to an Instance pointer used to query the properties of a physical device.
    /// - `physical_device_index`: The index of the physical device we are trying to get info from. Only used for debugging purposes.
    /// - `physical_device_name`: The name of the physical device we are trying to get info from. Only used for debugging purposes.
    /// 
    /// # Returns
    /// The new QueueFamilyInfo struct on success, or else a QueueError::OperationNotSupported error if the given device does not support all required queue family types.
    pub(crate) fn new(instance: &Rc<Instance>, physical_device: vk::PhysicalDevice, physical_device_index: usize, physical_device_name: &str) -> Result<Self, QueueError> {
        // Prepare placeholders for the different queues
        let mut graphics : Option<(u32, usize)> = None;
        let mut memory   : Option<(u32, usize)> = None;
        let mut compute  : Option<(u32, usize)> = None;

        // Iterate over the queue families
        let families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (i, family) in families.iter().enumerate() {
            // We need at least one queue in each family, obviously
            if family.queue_count == 0 { continue; }

            // Count the number of operations this queue can do
            let mut n_operations = 0;
            let supports_graphics = if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) { n_operations += 1; true } else { false };
            let supports_memory   = if family.queue_flags.contains(vk::QueueFlags::TRANSFER) { n_operations += 1; true } else { false };
            let supports_compute  = if family.queue_flags.contains(vk::QueueFlags::COMPUTE) { n_operations += 1; true } else { false };

            // Note the queue on every slot it supports, except we already have a more specialized one
            if supports_graphics && (graphics.is_none() || n_operations < graphics.as_ref().unwrap().1) {
                graphics = Some((i as u32, n_operations));
            }
            if supports_memory && (memory.is_none() || n_operations < memory.as_ref().unwrap().1) {
                memory = Some((i as u32, n_operations));
            }
            if supports_compute && (compute.is_none() || n_operations < compute.as_ref().unwrap().1) {
                compute = Some((i as u32, n_operations));
            }
        }

        // If we didn't find one of the queues, error
        let graphics = match graphics {
            Some(graphics) => graphics.0,
            None           => { return Err(QueueError::OperationUnsupported{ index: physical_device_index, name: physical_device_name.to_string(), operation: vk::QueueFlags::GRAPHICS }); }
        };
        let memory = match memory {
            Some(memory) => memory.0,
            None         => { return Err(QueueError::OperationUnsupported{ index: physical_device_index, name: physical_device_name.to_string(), operation: vk::QueueFlags::TRANSFER }); }
        };
        let compute = match compute {
            Some(compute) => compute.0,
            None          => { return Err(QueueError::OperationUnsupported{ index: physical_device_index, name: physical_device_name.to_string(), operation: vk::QueueFlags::COMPUTE }); }
        };

        // Otherwise, we can populate ourselves!
        Ok(QueueFamilyInfo {
            graphics : graphics,
            memory   : memory,
            present  : graphics,
            compute  : compute,
        })
    }



    /// Returns an iterator over the **different** families in the QueueFamilyInfo.
    #[inline]
    pub fn unique(&self) -> QueueFamilyInfoUniqueIterator {
        QueueFamilyInfoUniqueIterator::new(self)
    }

    /// Returns the number of **different** families in the QueueFamilyInfo.
    pub fn unique_len(&self) -> usize {
        if self.graphics != self.memory && self.graphics != self.compute && self.memory != self.compute {
            3
        } else if self.graphics != self.memory || self.graphics != self.compute || self.memory != self.compute {
            2
        } else {
            1
        }
    }



    /// Returns the queue index of the given QueueKind.
    #[inline]
    pub fn get_index(&self, kind: QueueKind) -> u32 {
        match kind {
            QueueKind::Graphics => self.graphics,
            QueueKind::Memory   => self.memory,
            QueueKind::Present  => self.present,
            QueueKind::Compute  => self.compute,
        }
    }
}



/// Implements an iterator over the unique family indices in the QueueFamilyInfo.
#[derive(Debug)]
pub struct QueueFamilyInfoUniqueIterator<'a> {
    /// The QueueFamilyInfo over which we iterate
    family_info : &'a QueueFamilyInfo,
    /// The current 'position' in the family info
    index       : usize,
}

impl<'a> QueueFamilyInfoUniqueIterator<'a> {
    /// Constructor for the QueueFamilyInfoUniqueIterator.
    /// 
    /// Prepares a new iterator over the given QueueFamilyInfo.
    /// 
    /// Note that it's passed by reference, so it's probably not a good idea to modify queue families while iterating over them.
    #[inline]
    pub(crate) fn new(family_info: &'a QueueFamilyInfo) -> Self {
        Self {
            family_info,
            index : 0,
        }
    }
}

impl<'a> Iterator for QueueFamilyInfoUniqueIterator<'a> {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Match based on the index
        match self.index {
            0 => { self.index += 1; Some(self.family_info.graphics) },
            1 => {
                // Only do this one if it's unique
                self.index += 1;
                if self.family_info.memory != self.family_info.graphics {
                    Some(self.family_info.memory)
                } else {
                    // Skip to the next value
                    self.next()
                }
            },
            2 => {
                // Only do this one if it's unique
                self.index += 1;
                if self.family_info.compute != self.family_info.graphics && self.family_info.compute != self.family_info.memory {
                    Some(self.family_info.compute)
                } else {
                    // Skip to the next value
                    self.next()
                }
            }
            _ => None,
        }
    }
}





/***** SURFACES *****/
/// Collects information about the SwapchainSupport for this device.
#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    /// Lists the capabilities of the chosen device/surface combo.
    pub capabilities  : vk::SurfaceCapabilitiesKHR,
    /// Lists the formats supported by the chosen device/surface combo.
    pub formats       : Vec<vk::SurfaceFormatKHR>,
    /// Lists the present modes supported by the chosen device/surface combo.
    pub present_modes : Vec<vk::PresentModeKHR>,
}





/***** DESCRIPTOR SETS / LAYOUTS *****/
/// Defines a single binding for the DescriptorSetLayout
#[derive(Clone, Debug)]
pub struct DescriptorBinding {
    /// The binding index of this binding (for use in shaders).
    pub binding : u32,
    /// The type of this binding.
    pub kind    : DescriptorKind,
    /// The shader stage where this binding is bound to.
    pub stage   : ShaderStage,
    /// The number of descriptors in this binding.
    pub count   : u32,
}

impl From<vk::DescriptorSetLayoutBinding> for DescriptorBinding {
    #[inline]
    fn from(value: vk::DescriptorSetLayoutBinding) -> Self {
        // Use the reference one instead
        Self::from(&value)
    }
}

impl From<&vk::DescriptorSetLayoutBinding> for DescriptorBinding {
    #[inline]
    fn from(value: &vk::DescriptorSetLayoutBinding) -> Self {
        Self {
            binding : value.binding,
            kind    : value.descriptor_type.into(),
            stage   : value.stage_flags.into(),
            count   : value.descriptor_count,
        }
    }
}

impl From<DescriptorBinding> for vk::DescriptorSetLayoutBinding {
    #[inline]
    fn from(value: DescriptorBinding) -> Self {
        // Use the reference one instead
        Self::from(&value)
    }
}

impl From<&DescriptorBinding> for vk::DescriptorSetLayoutBinding {
    #[inline]
    fn from(value: &DescriptorBinding) -> Self {
        Self {
            binding              : value.binding,
            descriptor_type      : value.kind.into(),
            stage_flags          : value.stage.into(),
            descriptor_count     : value.count,
            p_immutable_samplers : ptr::null(),
        }
    }
}





/***** RENDER PASSES *****/
/// Describes a single attachment
#[derive(Clone, Debug)]
pub struct AttachmentDescription {
    /// The format of the attachment.
    pub format  : ImageFormat,
    /// The number of samples to take of this image.
    pub samples : SampleCount,

    /// Defines what to do when loading this attachment (both for the colour _and_ depth aspects).
    pub on_load  : AttachmentLoadOp,
    /// Defines what to do with the pixels generated during the render pass (both for the colour _and_ depth aspects).
    pub on_store : AttachmentStoreOp,

    /// Defines what to do with any stencil attachment (if present).
    pub on_stencil_load  : AttachmentLoadOp,
    /// Defines what to do with the stencil values generated during the render pass (if present).
    pub on_stencil_store : AttachmentStoreOp,

    /// Define the layout of the attachment before the render pass (should match the previous).
    pub start_layout : ImageLayout,
    /// Define the layout of the attachment after the render pass (may be anything, will transition automatically).
    pub end_layout   : ImageLayout,
}

impl From<vk::AttachmentDescription> for AttachmentDescription {
    #[inline]
    fn from(value: vk::AttachmentDescription) -> Self {
        // Use the reference edition
        Self::from(&value)
    }
}

impl From<&vk::AttachmentDescription> for AttachmentDescription {
    #[inline]
    fn from(value: &vk::AttachmentDescription) -> Self {
        Self {
            format  : value.format.into(),
            samples : value.samples.into(),

            on_load  : value.load_op.into(),
            on_store : value.store_op.into(),

            on_stencil_load  : value.stencil_load_op.into(),
            on_stencil_store : value.stencil_store_op.into(),

            start_layout : value.initial_layout.into(),
            end_layout   : value.final_layout.into(),
        }
    }
}

impl From<AttachmentDescription> for vk::AttachmentDescription {
    #[inline]
    fn from(value: AttachmentDescription) -> Self {
        // Use the reference edition
        Self::from(&value)
    }
}

impl From<&AttachmentDescription> for vk::AttachmentDescription {
    #[inline]
    fn from(value: &AttachmentDescription) -> Self {
        Self {
            // Do the default stuff
            flags : vk::AttachmentDescriptionFlags::empty(),

            // Set some image attachment properties
            format  : value.format.into(),
            samples : value.samples.into(),

            // Define what to do when loading and storing this attachment
            load_op  : value.on_load.into(),
            store_op : value.on_store.into(),

            // Define what to do when loading and storing the stencil part of this attachment
            stencil_load_op  : value.on_stencil_load.into(),
            stencil_store_op : value.on_stencil_store.into(),

            initial_layout : value.start_layout.into(),
            final_layout   : value.end_layout.into(),
        }
    }
}



/// References an attachment.
#[derive(Clone, Debug)]
pub struct AttachmentRef {
    /// The index of the attachment to reference.
    pub index  : u32,
    /// The layout of the attachment at the time this reference is used (will be transitioned appropriately).
    pub layout : ImageLayout,
}

impl From<vk::AttachmentReference> for AttachmentRef {
    #[inline]
    fn from(value: vk::AttachmentReference) -> Self {
        // Simply use the reference version
        Self::from(&value)
    }
}

impl From<&vk::AttachmentReference> for AttachmentRef {
    #[inline]
    fn from(value: &vk::AttachmentReference) -> Self {
        Self {
            index  : value.attachment,
            layout : value.layout.into(),
        }
    }
}

impl From<AttachmentRef> for vk::AttachmentReference {
    #[inline]
    fn from(value: AttachmentRef) -> Self {
        // Simply use the reference version
        Self::from(&value)
    }
}

impl From<&AttachmentRef> for vk::AttachmentReference {
    #[inline]
    fn from(value: &AttachmentRef) -> Self {
        Self {
            attachment : value.index,
            layout     : value.layout.into(),
        }
    }
}



/// Describes a single subpass
#[derive(Clone, Debug)]
pub struct SubpassDescription {
    /// The bind point for this subpass (i.e., whether graphics or compute).
    pub bind_point : BindPoint,

    /// The input attachments for this subpass.
    pub input_attaches    : Vec<AttachmentRef>,
    /// The colour attachments for this subpass.
    pub colour_attaches   : Vec<AttachmentRef>,
    /// Any resolve attachments for this subpass. This array should have the same length as the colour attachments.
    pub resolve_attaches  : Vec<AttachmentRef>,
    /// Any attachments that are not used by this subpass, but must be passed to future subpasses.
    /// 
    /// To that end, only describes the indices for these attachments.
    pub preserve_attaches : Vec<u32>,

    /// The depth stencil attachment for this subpass.__rust_force_expr!
    pub depth_stencil : Option<AttachmentRef>,
}

impl From<vk::SubpassDescription> for SubpassDescription {
    fn from(value: vk::SubpassDescription) -> Self {
        // Cast the vectors and such to the appropriate Game types
        let input_attaches: Vec<AttachmentRef>   = unsafe { slice::from_raw_parts(value.p_input_attachments, value.input_attachment_count as usize) }.iter().map(|attach_ref| attach_ref.into()).collect();
        let colour_attaches: Vec<AttachmentRef>  = unsafe { slice::from_raw_parts(value.p_color_attachments, value.color_attachment_count as usize) }.iter().map(|attach_ref| attach_ref.into()).collect();
        let resolve_attaches: Vec<AttachmentRef> = unsafe { slice::from_raw_parts(value.p_resolve_attachments, value.color_attachment_count as usize) }.iter().map(|attach_ref| attach_ref.into()).collect();
        let preserve_attaches: Vec<u32>          = unsafe { slice::from_raw_parts(value.p_preserve_attachments, value.preserve_attachment_count as usize) }.to_vec();
        let depth_stencil: Option<AttachmentRef> = unsafe {
            // Switch between pointer value and non-pointer value
            if value.p_depth_stencil_attachment != ptr::null() {
                Some(value.p_depth_stencil_attachment.as_ref().expect("Could not unpack raw VkAttachmentReference pointer in VkSubpassDescription::p_depth_stencil_attachment").into())
            } else {
                None
            }
        };

        // Use those to initialize the Self
        Self {
            bind_point : value.pipeline_bind_point.into(),

            input_attaches,
            colour_attaches,
            resolve_attaches,
            preserve_attaches,

            depth_stencil,
        }
    }
}

impl Into<(vk::SubpassDescription, (Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<u32>, Option<Box<vk::AttachmentReference>>))> for SubpassDescription {
    /// Converts the ColourBlendState into a VkPipelineColorBlendStateCreateInfo.
    /// 
    /// However, due to the external references made in the VkPipelineColorBlendStateCreateInfo struct, it also returns one Vec that manages the external memory referenced.
    /// 
    /// # Returns
    /// A tuple with:
    /// - The new VkSubpassDescription instance
    /// - A tuple with the referenced memory:
    ///   - A vector with the input attachments
    ///   - A vector with the colour attachments
    ///   - A vector with the resolve attachments (same length as the colour attachments)
    ///   - A vector with the preserve attachments (as unsigned integers)
    ///   - A box with the depth stencil attachment
    fn into(self) -> (vk::SubpassDescription, (Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<vk::AttachmentReference>, Vec<u32>, Option<Box<vk::AttachmentReference>>)) {
        // Cast the vectors of self to the appropriate type
        let input_attaches: Vec<vk::AttachmentReference>        = self.input_attaches.iter().map(|attach_ref| attach_ref.into()).collect();
        let colour_attaches: Vec<vk::AttachmentReference>       = self.colour_attaches.iter().map(|attach_ref| attach_ref.into()).collect();
        let resolve_attaches: Vec<vk::AttachmentReference>      = self.resolve_attaches.iter().map(|attach_ref| attach_ref.into()).collect();
        let preserve_attaches: Vec<u32>                         = self.preserve_attaches.clone();
        let depth_stencil: Option<Box<vk::AttachmentReference>> = self.depth_stencil.map(|attach_ref| Box::new(attach_ref.into()));

        // Create the VUlkan struct with the references
        let result = vk::SubpassDescription {
            // Do the default stuff
            flags : vk::SubpassDescriptionFlags::empty(),

            // Set the bind point
            pipeline_bind_point : self.bind_point.into(),

            // Set the input attachments
            input_attachment_count : input_attaches.len() as u32,
            p_input_attachments    : vec_as_ptr!(input_attaches),

            // Set the colour & associated resolve attachments
            color_attachment_count : colour_attaches.len() as u32,
            p_color_attachments    : vec_as_ptr!(colour_attaches),
            p_resolve_attachments  : vec_as_ptr!(resolve_attaches),

            // Set the preserve attachments
            preserve_attachment_count : preserve_attaches.len() as u32,
            p_preserve_attachments    : vec_as_ptr!(preserve_attaches),

            // Set the depth stencil
            p_depth_stencil_attachment : match depth_stencil.as_ref() {
                Some(depth_stencil) => &**depth_stencil,
                None                => ptr::null(),
            },
        };

        // Done - return it and its memory managers
        (result, (
            input_attaches,
            colour_attaches,
            resolve_attaches,
            preserve_attaches,
            depth_stencil,
        ))
    }
}



/// Describes a dependency between two subpasses
#[derive(Clone, Debug)]
pub struct SubpassDependency {
    /// The index of the subpass that is the one we transition from.
    pub from : u32,
    /// The index of the subpass that is the one we transition to.
    pub to   : u32,

    /// The stage flags of the transition-from subpass where the dependency is relevant.
    pub from_stage : PipelineStage,
    /// The stage flags of the transition-to subpass where the dependency is relevant.
    pub to_stage   : PipelineStage,

    /// The kind of operation(s) that the first subpass has to complete before we can move to the second.
    pub from_access : AccessFlags,
    /// The kind of operation(s) that the second subpass cannot do until the operations of the first subpass are completed.
    pub to_access   : AccessFlags,

    /// Any other dependency flags that are relevant for this transaction.
    pub dependency_flags : DependencyFlags,
}

impl From<vk::SubpassDependency> for SubpassDependency {
    #[inline]
    fn from(value: vk::SubpassDependency) -> Self {
        // Simply call the reference version
        Self::from(&value)
    }
}

impl From<&vk::SubpassDependency> for SubpassDependency {
    #[inline]
    fn from(value: &vk::SubpassDependency) -> Self {
        Self {
            from : value.src_subpass,
            to   : value.dst_subpass,

            from_stage : value.src_stage_mask.into(),
            to_stage   : value.dst_stage_mask.into(),

            from_access : value.src_access_mask.into(),
            to_access   : value.dst_access_mask.into(),

            dependency_flags : value.dependency_flags.into(),
        }
    }
}

impl From<SubpassDependency> for vk::SubpassDependency {
    #[inline]
    fn from(value: SubpassDependency) -> Self {
        // Simply call the reference version
        Self::from(&value)
    }
}

impl From<&SubpassDependency> for vk::SubpassDependency {
    #[inline]
    fn from(value: &SubpassDependency) -> Self {
        Self {
            src_subpass : value.from,
            dst_subpass : value.to,

            src_stage_mask : value.from_stage.into(),
            dst_stage_mask : value.to_stage.into(),

            src_access_mask : value.from_access.into(),
            dst_access_mask : value.to_access.into(),

            dependency_flags : value.dependency_flags.into(),
        }
    }
}





/***** PIPELINES *****/
/// Defines how a single attribute (i.e., field in the Vertex struct) looks like.
#[derive(Clone, Debug)]
pub struct VertexAttribute {
    /// The location in the shader of this attribute (must be arbitrary but unique).
    pub location : u32,
    /// The binding (i.e., the Vertex buffer) where this attribute's vertex lives
    pub binding  : u32,
    /// Describes the byte layout of this attribute.
    pub layout   : AttributeLayout,
    /// Notes where to find the attribute in the parent Vertex struct (offset as bytes)
    pub offset   : usize,
}

impl From<vk::VertexInputAttributeDescription> for VertexAttribute {
    #[inline]
    fn from(value: vk::VertexInputAttributeDescription) -> Self {
        // Use the reference version
        Self::from(&value)
    }
}

impl From<&vk::VertexInputAttributeDescription> for VertexAttribute {
    #[inline]
    fn from(value: &vk::VertexInputAttributeDescription) -> Self {
        // Simply populate the VertexAttribute via its struct interface
        Self {
            location : value.location,
            binding  : value.binding,
            layout   : match value.format.try_into() {
                Ok(layout) => layout,
                Err(err)   => { panic!("Illegal attribute format: {}", err); }
            },
            offset   : value.offset as usize,
        }
    }
}

impl From<VertexAttribute> for vk::VertexInputAttributeDescription {
    #[inline]
    fn from(value: VertexAttribute) -> Self {
        // Use the reference version
        Self::from(&value)
    }
}

impl From<&VertexAttribute> for vk::VertexInputAttributeDescription {
    #[inline]
    fn from(value: &VertexAttribute) -> Self {
        // Simply populate the VertexInputAttributeDescription via its struct interface
        Self {
            location : value.location,
            binding  : value.binding,
            format   : value.layout.into(),
            offset   : value.offset as u32,
        }
    }
}



/// Defines how a single binding (i.e., list of vectors) looks like.
#[derive(Clone, Debug)]
pub struct VertexBinding {
    /// The binding index of this buffer
    pub binding : u32,
    /// The stride, i.e., size of each vertex
    pub stride  : usize,
    /// The input rate of the vertices. Concretely, this is either a function of an index or directly reading the vertices from the buffer.
    pub rate    : VertexInputRate,
}

impl From<vk::VertexInputBindingDescription> for VertexBinding {
    #[inline]
    fn from(value: vk::VertexInputBindingDescription) -> Self {
        // Use the reference version
        Self::from(&value)
    }
}

impl From<&vk::VertexInputBindingDescription> for VertexBinding {
    #[inline]
    fn from(value: &vk::VertexInputBindingDescription) -> Self {
        // Simply populate the VertexBinding via its struct interface
        Self {
            binding : value.binding,
            stride  : value.stride as usize,
            rate    : value.input_rate.into(),
        }
    }
}

impl From<VertexBinding> for vk::VertexInputBindingDescription {
    #[inline]
    fn from(value: VertexBinding) -> Self {
        // Use the reference version
        Self::from(&value)
    }
}

impl From<&VertexBinding> for vk::VertexInputBindingDescription {
    #[inline]
    fn from(value: &VertexBinding) -> Self {
        // Simply populate the VertexInputBindingDescription via its struct interface
        Self {
            binding    : value.binding,
            stride     : value.stride as u32,
            input_rate : value.rate.into(),
        }
    }
}



/// Defines the layout of the input vertices given to the pipeline.
#[derive(Clone, Debug)]
pub struct VertexInputState {
    /// A list of attributes (as VertexAttribute) of each incoming vertex.
    pub attributes : Vec<VertexAttribute>,
    /// A list of bindings (as VertexBinding) of the different Vertex buffers.
    pub bindings   : Vec<VertexBinding>,
}

impl From<&vk::PipelineVertexInputStateCreateInfo> for VertexInputState {
    fn from(value: &vk::PipelineVertexInputStateCreateInfo) -> Self {
        // Create the two vectors with copies from the vertex attributes
        let attributes: &[vk::VertexInputAttributeDescription] = unsafe { slice::from_raw_parts(value.p_vertex_attribute_descriptions, value.vertex_attribute_description_count as usize) };
        let bindings: &[vk::VertexInputBindingDescription]     = unsafe { slice::from_raw_parts(value.p_vertex_binding_descriptions, value.vertex_binding_description_count as usize) };

        // Copy the vectors to our structs
        let attributes: Vec<VertexAttribute> = attributes.iter().map(|attr| attr.into()).collect();
        let bindings: Vec<VertexBinding>     = bindings.iter().map(|bind| bind.into()).collect();

        // Return the new instance with these vectors
        Self {
            attributes,
            bindings,
        }
    }
}

impl Into<(vk::PipelineVertexInputStateCreateInfo, (Vec<vk::VertexInputAttributeDescription>, Vec<vk::VertexInputBindingDescription>))> for VertexInputState {
    /// Converts the VertexInputState into a VkPipelineVertexInputStateCreateInfo.
    /// 
    /// However, due to the external references made in the VkPipelineVertexInputStateCreateInfo struct, it also returns two vectors that manage the external memory referenced.
    /// 
    /// # Returns
    /// A tuple with:
    /// - The new VkPipelineVertexInputStateCreateInfo instance
    /// - A tuple with:
    ///   - The vector with the attributes
    ///   - The vector with the bindings
    fn into(self) -> (vk::PipelineVertexInputStateCreateInfo, (Vec<vk::VertexInputAttributeDescription>, Vec<vk::VertexInputBindingDescription>)) {
        // Cast the vectors to their Vulkan counterparts
        let attributes: Vec<vk::VertexInputAttributeDescription> = self.attributes.iter().map(|attr| attr.into()).collect();
        let bindings: Vec<vk::VertexInputBindingDescription>     = self.bindings.iter().map(|bind| bind.into()).collect();

        // Create the new instance with these vectors
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            // Do the standard stuff
            s_type : vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineVertexInputStateCreateFlags::empty(),

            // Add the attributes
            vertex_attribute_description_count : attributes.len() as u32,
            p_vertex_attribute_descriptions    : vec_as_ptr!(attributes),

            // Add the bindings
            vertex_binding_description_count : bindings.len() as u32,
            p_vertex_binding_descriptions    : vec_as_ptr!(bindings),
        };

        // Return the struct with its memory managers
        (vertex_input_state, (attributes, bindings))
    }
}



/// Defines how to construct primitives from the input vertices.
#[derive(Clone, Debug)]
pub struct VertexAssemblyState {
    /// The topology of the input vertices
    pub topology          : VertexTopology,
    /// Whether or not a special vertex value is reserved for resetting a primitive mid-way
    pub restart_primitive : bool,
}

impl From<vk::PipelineInputAssemblyStateCreateInfo> for VertexAssemblyState {
    #[inline]
    fn from(value: vk::PipelineInputAssemblyStateCreateInfo) -> Self {
        // Simply use the default struct constructor
        Self {
            topology          : value.topology.into(),
            restart_primitive : value.primitive_restart_enable != 0,
        }
    }
}

impl From<VertexAssemblyState> for vk::PipelineInputAssemblyStateCreateInfo {
    #[inline]
    fn from(value: VertexAssemblyState) -> Self {
        // Simply use the default struct constructor
        Self {
            // Do the default stuff
            s_type : vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineInputAssemblyStateCreateFlags::empty(),

            // Set the topology and the bool
            topology                 : value.topology.into(),
            primitive_restart_enable : value.restart_primitive as u32,
        }
    }
}



/// Defines the dimensions of a resulting frame.
#[derive(Clone, Debug)]
pub struct ViewportState {
    /// The rectangle that defines the viewport's dimensions.
    /// 
    /// Note that this will actually be ignored if the viewport is given as a dynamic state.
    pub viewport : Rect2D<f32>,
    /// The rectangle that defines any cutoff to the viewport.
    /// 
    /// Note that this will actually be ignored if the scissor is given as a dynamic state.
    pub scissor  : Rect2D<i32, u32>,
    /// The depth range of the Viewport. Anything that falls outside of it will be clipped.
    pub depth    : Range<f32>,
}

impl From<&vk::PipelineViewportStateCreateInfo> for ViewportState {
    #[inline]
    fn from(value: &vk::PipelineViewportStateCreateInfo) -> Self {
        // Make sure the viewport state does not use multiple viewports / scissors
        if value.viewport_count != 1 || value.scissor_count != 1 { panic!("Encountered VkPipelineViewportStateCreateInfo with multiple viewports and/or scissors"); }

        // Fetch the only viewport and scissor
        let viewport: vk::Viewport = unsafe { slice::from_raw_parts(value.p_viewports, 1) }[0];
        let scissor: vk::Rect2D    = unsafe { slice::from_raw_parts(value.p_scissors, 1) }[0];

        // Use the default constructor syntax
        Self {
            viewport : Rect2D::new(viewport.x, viewport.y, viewport.width, viewport.height),
            scissor  : scissor.into(),
            depth    : viewport.min_depth..viewport.max_depth,
        }
    }
}

impl Into<(vk::PipelineViewportStateCreateInfo, (Box<vk::Viewport>, Box<vk::Rect2D>))> for ViewportState {
    /// Converts the Viewport into a VkPipelineViewportStateCreateInfo.
    /// 
    /// However, due to the external references made in the VkPipelineViewportStateCreateInfo struct, it also returns two Boxes that manage the external memory referenced.
    /// 
    /// # Returns
    /// A tuple with:
    /// - The new VkPipelineViewportStateCreateInfo instance
    /// - A tuple with:
    ///   - The Box with the viewport
    ///   - The Box with the scissor
    fn into(self) -> (vk::PipelineViewportStateCreateInfo, (Box<vk::Viewport>, Box<vk::Rect2D>)) {
        // Cast the viewport and scissor to their Vulkan counterparts
        let viewport: Box<vk::Viewport> = Box::new(vk::Viewport {
            x         : self.viewport.x(),
            y         : self.viewport.y(),
            width     : self.viewport.w(),
            height    : self.viewport.h(),
            min_depth : self.depth.start,
            max_depth : self.depth.end,
        });
        let scissor: Box<vk::Rect2D> = Box::new(self.scissor.into());

        // Put the pointers in the new struct to return
        let result = vk::PipelineViewportStateCreateInfo {
            // Set the standard fields
            s_type : vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineViewportStateCreateFlags::empty(),
            
            // Set the only viewport
            viewport_count : 1,
            p_viewports    : &*viewport,

            // Set the only scissor
            scissor_count : 1,
            p_scissors    : &*scissor,
        };

        // Now return the new struct plus its memory manages
        (result, (viewport, scissor))
    }
}

impl From<ViewportState> for vk::Viewport {
    fn from(value: ViewportState) -> Self {
        // Use the default constructor syntax
        Self {
            x         : value.viewport.x(),
            y         : value.viewport.y(),
            width     : value.viewport.w(),
            height    : value.viewport.h(),
            min_depth : value.depth.start,
            max_depth : value.depth.end,
        }
    }
}



/// Defines the fixed rasterization stage for a Pipeline.
#[derive(Clone, Debug)]
pub struct RasterizerState {
    /// Defines the culling mode for the Rasterization stage
    pub cull_mode  : CullMode,
    /// Defines which winding direction we consider to be 'front'
    pub front_face : FrontFace,

    /// Defines the thickness of the lines drawn by the rasterizer. Note, though, that anything larger than a 1.0f requires a GPU feature
    pub line_width : f32,
    /// Defines what to draw in between the vertices
    pub draw_mode  : DrawMode,

    /// Whether or not to discard the fragments after the rasterizer (lol)
    pub discard_result : bool,

    /// Whether to enable depth clamping or not (i.e., clamping objects to a certain depth)
    pub depth_clamp : bool,
    /// The value to clamp the depth to (whether upper or lower depends on testing op used)
    pub clamp_value : f32,

    /// Whether or not to change the depth value before testing and writing
    pub depth_bias   : bool,
    /// The factor of depth to apply to the depth of each fragment (i.e., scaling)
    pub depth_factor : f32,
    /// The factor to apply to the slope(?) of the fragment during depth calculation
    pub depth_slope  : f32,
}

impl From<vk::PipelineRasterizationStateCreateInfo> for RasterizerState {
    #[inline]
    fn from(value: vk::PipelineRasterizationStateCreateInfo) -> Self {
        // Simply use the default construction syntax
        Self {
            cull_mode  : value.cull_mode.into(),
            front_face : value.front_face.into(),

            line_width : value.line_width,
            draw_mode  : value.polygon_mode.into(),

            discard_result : value.rasterizer_discard_enable != 0,

            depth_clamp : value.depth_clamp_enable != 0,
            clamp_value : value.depth_bias_clamp,

            depth_bias   : value.depth_bias_enable != 0,
            depth_factor : value.depth_bias_constant_factor,
            depth_slope  : value.depth_bias_slope_factor,
        }
    }
}

impl From<RasterizerState> for vk::PipelineRasterizationStateCreateInfo {
    #[inline]
    fn from(value: RasterizerState) -> Self {
        // Simply use the default construction syntax
        Self {
            // Set the default flags
            s_type : vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineRasterizationStateCreateFlags::empty(),

            // Set the culling mode & associated front face
            cull_mode  : value.cull_mode.into(),
            front_face : value.front_face.into(),
            
            // Set how to draw the fragments
            line_width   : value.line_width,
            polygon_mode : value.draw_mode.into(),

            // Determine whether to keep everything or not (invert that)
            rasterizer_discard_enable : value.discard_result as u32,

            // Set the depth clamp stuff
            depth_clamp_enable : value.depth_clamp as u32,
            depth_bias_clamp   : value.clamp_value,

            // Set the depth bias stuff
            depth_bias_enable          : value.depth_bias as u32,
            depth_bias_constant_factor : value.depth_factor,
            depth_bias_slope_factor    : value.depth_slope,
        }
    }
}



/// Defines if and how to multisample for a Pipeline
#[derive(Clone, Debug)]
pub struct MultisampleState {}

impl From<vk::PipelineMultisampleStateCreateInfo> for MultisampleState {
    #[inline]
    fn from(_value: vk::PipelineMultisampleStateCreateInfo) -> Self {
        Self {}
    }
}

impl From<MultisampleState> for vk::PipelineMultisampleStateCreateInfo {
    #[inline]
    fn from(_value: MultisampleState) -> Self {
        Self {
            // Set the default values
            s_type : vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineMultisampleStateCreateFlags::empty(),
            
            // Set the number of samples
            rasterization_samples : vk::SampleCountFlags::TYPE_1,

            // Set whether to shade the samples
            sample_shading_enable : vk::FALSE,
            min_sample_shading    : 0.0,

            // Set a possible mask for the different samples
            p_sample_mask : ptr::null(),

            // Set some alpha properties for the samples
            alpha_to_one_enable      : vk::FALSE,
            alpha_to_coverage_enable : vk::FALSE,
        }
    }
}



/// Defines how to interact with a given stencil.
#[derive(Clone, Debug)]
pub struct StencilOpState {
    /// Defines what to do if the stencil test fails
    pub on_stencil_fail : StencilOp,
    /// Defines what to do if the depth test fails
    pub on_depth_fail   : StencilOp,
    /// Defines what to do if the stencil test and depth test succeed
    pub on_success      : StencilOp,

    /// Defines the operator used in the stencil test
    pub compare_op   : CompareOp,
    /// Defines the mask to apply to value that are considered during the test
    pub compare_mask : u32,
    /// Defines the mask to apply when writing a victorious value
    pub write_mask   : u32,
    /// The integer reference that is used during the stencil test
    pub reference    : u32,
}

impl From<vk::StencilOpState> for StencilOpState {
    #[inline]
    fn from(value: vk::StencilOpState) -> Self {
        Self {
            on_stencil_fail : value.fail_op.into(),
            on_depth_fail   : value.depth_fail_op.into(),
            on_success      : value.pass_op.into(),

            compare_op   : value.compare_op.into(),
            compare_mask : value.compare_mask,
            write_mask   : value.write_mask,
            reference    : value.reference,
        }
    }
}

impl From<StencilOpState> for vk::StencilOpState {
    #[inline]
    fn from(value: StencilOpState) -> Self {
        Self {
            fail_op       : value.on_stencil_fail.into(),
            depth_fail_op : value.on_depth_fail.into(),
            pass_op       : value.on_success.into(),

            compare_op   : value.compare_op.into(),
            compare_mask : value.compare_mask,
            write_mask   : value.write_mask,
            reference    : value.reference,
        }
    }
}




/// Defines if a depth stencil is present in the Pipeline and how.
#[derive(Clone, Debug)]
pub struct DepthTestingState {
    /// Whether to enable depth testing
    pub enable_depth   : bool,
    /// Whether to enable depth writing (only if `enable_depth` is true).
    pub enable_write   : bool,
    /// Whether to enable normal stencil testing
    pub enable_stencil : bool,
    /// Whether to enable depth bounds testing
    pub enable_bounds : bool,

    /// The compare operation to perform when testing the depth
    pub compare_op    : CompareOp,

    /// The properties of the stencil test before the depth testing
    pub pre_stencil_test : StencilOpState,
    /// The properties of the stencil test after the depth testing
    pub post_stencil_test : StencilOpState,

    /// The minimum depth bound used in the depth bounds test
    pub min_bound : f32,
    /// The maximum depth bound used in the depth bounds test
    pub max_bound : f32,
}

impl From<vk::PipelineDepthStencilStateCreateInfo> for DepthTestingState {
    #[inline]
    fn from(value: vk::PipelineDepthStencilStateCreateInfo) -> Self {
        Self {
            enable_depth   : value.depth_test_enable != 0,
            enable_write   : value.depth_write_enable != 0,
            enable_stencil : value.stencil_test_enable != 0,
            enable_bounds  : value.depth_bounds_test_enable != 0,

            compare_op : value.depth_compare_op.into(),

            pre_stencil_test  : value.front.into(),
            post_stencil_test : value.back.into(),

            min_bound : value.min_depth_bounds,
            max_bound : value.max_depth_bounds,
        }
    }
}

impl From<DepthTestingState> for vk::PipelineDepthStencilStateCreateInfo {
    #[inline]
    fn from(value: DepthTestingState) -> Self {
        Self {
            // Do the default stuff
            s_type : vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineDepthStencilStateCreateFlags::empty(),

            // Define which tests to enable
            depth_test_enable        : value.enable_depth as u32,
            depth_write_enable       : value.enable_write as u32,
            stencil_test_enable      : value.enable_stencil as u32,
            depth_bounds_test_enable : value.enable_bounds as u32,

            // Define the compare operation for the depth test
            depth_compare_op : value.compare_op.into(),

            // Define the stencil test states
            front : value.pre_stencil_test.into(),
            back  : value.post_stencil_test.into(),

            // Define the bounds for the bounds test
            min_depth_bounds : value.min_bound,
            max_depth_bounds : value.max_bound,
        }
    }
}



/// Defines how to write colours to a single colour attachment.
#[derive(Clone, Debug)]
pub struct AttachmentBlendState {
    /// Whether to enable blending or not (values pass through unmodified if false).
    pub enable_blend : bool,

    /// The proportion of colour we take from the source.
    pub src_colour : BlendFactor,
    /// The proportion of colour we take from the destination.
    pub dst_colour : BlendFactor,
    /// The operator to use to blend the two colours
    pub colour_op  : BlendOp,

    /// The proportion of alpha we take from the source.
    pub src_alpha : BlendFactor,
    /// The proportion of alpha we take from the destination.
    pub dst_alpha : BlendFactor,
    /// The operator to use to blend the two alphas
    pub alpha_op  : BlendOp,

    /// A mask that specifies which channels are available for writing the blend.
    pub write_mask : ColourComponentFlags,
}

impl From<vk::PipelineColorBlendAttachmentState> for AttachmentBlendState {
    #[inline]
    fn from(value: vk::PipelineColorBlendAttachmentState) -> Self {
        // Use the reference version
        Self::from(&value)
    }
}

impl From<&vk::PipelineColorBlendAttachmentState> for AttachmentBlendState {
    #[inline]
    fn from(value: &vk::PipelineColorBlendAttachmentState) -> Self {
        Self {
            enable_blend : value.blend_enable != 0,

            src_colour : value.src_color_blend_factor.into(),
            dst_colour : value.dst_color_blend_factor.into(),
            colour_op  : value.color_blend_op.into(),

            src_alpha : value.src_alpha_blend_factor.into(),
            dst_alpha : value.dst_alpha_blend_factor.into(),
            alpha_op  : value.alpha_blend_op.into(),

            write_mask : value.color_write_mask.into(),
        }
    }
}

impl From<AttachmentBlendState> for vk::PipelineColorBlendAttachmentState {
    #[inline]
    fn from(value: AttachmentBlendState) -> Self {
        // Use the reference version
        Self::from(&value)
    }
}

impl From<&AttachmentBlendState> for vk::PipelineColorBlendAttachmentState {
    #[inline]
    fn from(value: &AttachmentBlendState) -> Self {
        Self {
            blend_enable : value.enable_blend as u32,

            src_color_blend_factor : value.src_colour.into(),
            dst_color_blend_factor : value.dst_colour.into(),
            color_blend_op         : value.colour_op.into(),

            src_alpha_blend_factor : value.src_alpha.into(),
            dst_alpha_blend_factor : value.dst_alpha.into(),
            alpha_blend_op         : value.alpha_op.into(),

            color_write_mask : value.write_mask.into(),
        }
    }
}



/// Defines how to write colours to the (multiple) colour attachments.
#[derive(Clone, Debug)]
pub struct ColourBlendState {
    /// Whether to apply any logic operations for all attachments.
    /// 
    /// If set to true, then ignores the attachment operations.
    pub enable_logic : bool,
    /// The logic operator to apply, if enabled.
    pub logic_op     : LogicOp,

    /// The list of colour attachment blend states that describe the per-attachment stats.
    pub attachment_states : Vec<AttachmentBlendState>,
    /// The constants for blending.
    pub blend_constants   : [f32; 4],
}

impl From<&vk::PipelineColorBlendStateCreateInfo> for ColourBlendState {
    fn from(value: &vk::PipelineColorBlendStateCreateInfo) -> Self {
        // Collect the raw pointers in a slice
        let attachments = unsafe { slice::from_raw_parts(value.p_attachments, value.attachment_count as usize) };

        // Cast them to our attachments, in a vec
        let attachments: Vec<AttachmentBlendState> = attachments.iter().map(|att| att.into()).collect();

        // Now create the struct with it and other properties
        Self {
            enable_logic : value.logic_op_enable != 0,
            logic_op     : value.logic_op.into(),

            attachment_states : attachments,
            blend_constants   : value.blend_constants.clone(),
        }
    }
}

impl Into<(vk::PipelineColorBlendStateCreateInfo, Vec<vk::PipelineColorBlendAttachmentState>)> for ColourBlendState {
    /// Converts the ColourBlendState into a VkPipelineColorBlendStateCreateInfo.
    /// 
    /// However, due to the external references made in the VkPipelineColorBlendStateCreateInfo struct, it also returns one Vec that manages the external memory referenced.
    /// 
    /// # Returns
    /// A tuple with:
    /// - The new VkPipelineColorBlendStateCreateInfo instance
    /// - The Vec with the referenced memory
    fn into(self) -> (vk::PipelineColorBlendStateCreateInfo, Vec<vk::PipelineColorBlendAttachmentState>) {
        // Cast our own attachment states to Vulkan's
        let attachments: Vec<vk::PipelineColorBlendAttachmentState> = self.attachment_states.iter().map(|att| att.into()).collect();

        // Now create the struct with it and other properties
        let result = vk::PipelineColorBlendStateCreateInfo {
            // Set the default stuff
            s_type : vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineColorBlendStateCreateFlags::empty(),

            // Set the logic properties
            logic_op_enable : self.enable_logic as u32,
            logic_op        : self.logic_op.into(),

            // Set the attachments and the blend constants
            attachment_count : attachments.len() as u32,
            p_attachments    : vec_as_ptr!(attachments),
            blend_constants  : self.blend_constants.clone(),
        };

        // Done, return both it and the memory
        (result, attachments)
    }
}



/***** MEMORY POOLS *****/
/// Defines the memory requirements of a buffer or image.
#[derive(Clone, Debug)]
pub struct MemoryRequirements {
    /// The minimum size of the required memory block.
    pub size  : usize,
    /// The alignment (in bytes) of the start of the required memory block. Must be a multiple of two.
    pub align : u64,
    /// The device memory types that are supported by the buffer or image for this particular usage.
    pub types : DeviceMemoryTypeFlags,
}

impl From<vk::MemoryRequirements> for MemoryRequirements {
    #[inline]
    fn from(value: vk::MemoryRequirements) -> Self {
        Self {
            size  : value.size as usize,
            align : value.alignment as u64,
            types : value.memory_type_bits.into(),
        }
    }
}

impl From<MemoryRequirements> for vk::MemoryRequirements {
    #[inline]
    fn from(value: MemoryRequirements) -> Self {
        Self {
            size             : value.size as vk::DeviceSize,
            alignment        : value.align as vk::DeviceSize,
            memory_type_bits : value.types.into(),
        }
    }
}



/// An auxillary struct that describes the memory requirements and properties of a given Buffer.
#[derive(Clone, Debug)]
pub struct BufferAllocateInfo {
    /// The usage flags of this Buffer
    pub usage_flags  : BufferUsageFlags,
    /// The sharing mode that determines how this Buffer may be accessed.
    pub sharing_mode : SharingMode,

    /// The size of the buffer, in bytes. Note that the resulting *actual* size may be slightly more based on alignment requirements.
    pub size         : usize,
    /// The additional properties that we require of the memory behind this buffer.
    pub memory_props : MemoryPropertyFlags,

    /// The type of allocator to use for this buffer.
    pub allocator : MemoryAllocatorKind,
}





/***** IMAGES *****/
/// Defines any potential re-mapping of an image's channels.
#[derive(Debug, Clone)]
pub struct ComponentMapping {
    /// The mapping of the red channel
    pub red   : ComponentSwizzle,
    /// The mapping of the green channel
    pub green : ComponentSwizzle,
    /// The mapping of the blue channel
    pub blue  : ComponentSwizzle,
    /// The mapping of the alpha channel
    pub alpha : ComponentSwizzle,
}

impl Default for ComponentMapping {
    fn default() -> Self {
        Self {
            red   : ComponentSwizzle::Identity,
            green : ComponentSwizzle::Identity,
            blue  : ComponentSwizzle::Identity,
            alpha : ComponentSwizzle::Identity,
        }
    }
}

impl From<vk::ComponentMapping> for ComponentMapping {
    #[inline]
    fn from(value: vk::ComponentMapping) -> Self {
        Self {
            red   : value.r.into(),
            green : value.g.into(),
            blue  : value.b.into(),
            alpha : value.a.into(),
        }
    }
}

impl From<ComponentMapping> for vk::ComponentMapping {
    #[inline]
    fn from(value: ComponentMapping) -> Self {
        Self {
            r : value.red.into(),
            g : value.green.into(),
            b : value.blue.into(),
            a : value.alpha.into(),
        }
    }
}
