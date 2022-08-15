//  ENUMS.rs
//    by Lut99
// 
//  Created:
//    09 Jul 2022, 12:23:22
//  Last edited:
//    15 Aug 2022, 17:55:13
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements various auxillary enums that are used throughout this
//!   crate
// 

use std::cmp::Ordering;
use std::ffi::CString;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr;

use ash::vk;

use crate::to_cstring;
use crate::errors::{AttributeLayoutError, ExtensionError};


/***** HELPER MACROS *****/
/// Implement the two-way from between the given Vulkan enum and ours.
macro_rules! enum_from {
    (impl From<vk::$from:ident> for $to:ident {
        $($match:ident => $target:ident $(,)?),+
        $(, _          => $rtarget:ident $(,)?)?
    }) => {
        enum_from!(impl From<vk::$from> for $to { $(vk::$from::$match => $to::$target),+ $(, _ => $to::$rtarget $(,)?)? });
    };

    (impl From<vk::$from:ident> for $to:ident {
        $($match:path => $target:ident $(,)?),+
        $(, _         => $rtarget:ident $(,)?)?
    }) => {
        enum_from!(impl From<vk::$from> for $to { $($match => $to::$target),+ $(, _ => $to::$rtarget $(,)?)? });
    };

    (impl From<vk::$from:ident> for $to:ident {
        $($match:ident => $target:path $(,)?),+
        $(, _          => $rtarget:path $(,)?)?
    }) => {
        enum_from!(impl From<vk::$from> for $to { $(vk::$from::$match => $target),+ $(, _ => $rtarget $(,)?)? });
    };

    (impl From<vk::$from:ident> for $to:ident {
        $($match:path => $target:path $(,)?),+
        $(,           => $rtarget:path $(,)?)?
    }) => {
        impl From<vk::$from> for $to {
            #[inline]
            fn from(value: vk::$from) -> Self {
                match value {
                    $($match => $target),+,
                    $(_      => $rtarget,)?
                    #[allow(unreachable_patterns)]
                    value    => { panic!(concat!("Encountered illegal value '{}' for ", stringify!(vk::$from)), value.as_raw()); }
                }
            }
        }

        impl From<$to> for vk::$from {
            #[inline]
            fn from(value: $to) -> Self {
                match value {
                    $($target => $match),+,
                    #[allow(unreachable_patterns)]
                    value     => { panic!(concat!("Encountered illegal value '{:?}' for ", stringify!($to)), value); }
                }
            }
        }
    };
}





/***** INSTANCE *****/
/// An enum that describes instance extensions used in the Game.
#[derive(Copy, Clone, Debug)]
pub enum InstanceExtension {
    /// The instance portability extension, used on macOS
    PortabilityEnumeration,
}

impl InstanceExtension {
    /// Constant function to get the string value of the InstanceExtension.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        use InstanceExtension::*;
        match self {
            PortabilityEnumeration => "VK_KHR_portability_enumeration",
        }
    }
}

impl Display for InstanceExtension {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.as_str())
    }
}

impl From<InstanceExtension> for CString {
    #[inline]
    fn from(value: InstanceExtension) -> Self {
        to_cstring!(format!("{}", value))
    }
}

impl FromStr for InstanceExtension {
    type Err = ExtensionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "VK_KHR_portability_enumeration" => Ok(InstanceExtension::PortabilityEnumeration),
            value                            => Err(ExtensionError::UnknownInstanceExtension{ got: value.into() }),
        }
    }
}



/// An enum that describes instance layers used in the Game.
#[derive(Copy, Clone, Debug)]
pub enum InstanceLayer {
    /// The Khronos validation layer
    KhronosValidation,
}

impl InstanceLayer {
    /// Constant function to get the string value of the InstanceLayer.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        use InstanceLayer::*;
        match self {
            KhronosValidation => "VK_LAYER_KHRONOS_validation",
        }
    }
}

impl Display for InstanceLayer {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for InstanceLayer {
    type Err = ExtensionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "VK_LAYER_KHRONOS_validation" => Ok(Self::KhronosValidation),
            value                         => Err(ExtensionError::UnknownInstanceLayer{ got: value.into() }),
        }
    }
}





/***** DEVICES *****/
/// Enumerates the possible Device types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceKind {
    /// A discrete GPU. Is given the highest 'CPU disconnectedness' score.
    Discrete,
    /// An intergrated but dedicated GPU. Is given the second highest 'CPU disconnectedness' score.
    Integrated,
    /// A Virtual GPU, which is given the third-worst 'CPU disconnectedness' score.
    Virtual,
    /// No dedicated GPU at all, just the CPU doing GPU stuff. Is given the fourth-worst 'CPU disconnectedness' score.
    Cpu,
    /// A GPU type which we do not know, which we prefer the least (worst 'CPU disconnectedness' score).
    Other,
}

impl DeviceKind {
    /// Returns a so-ca,lled 'CPU disconnectedness' score, which we hope to equate to a device's power when comparing multiple.
    /// 
    /// We assume that devices with a higher score are more discrete, and thus more powerful.
    /// 
    /// # Returns
    /// The score as an unsigned integer.
    #[inline]
    pub fn score(&self) -> u32 {
        use DeviceKind::*;
        match self {
            Discrete   => 4,
            Integrated => 3,
            Virtual    => 2,
            Cpu        => 1,
            Other      => 0,
        }
    }
}

impl Ord for DeviceKind {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by CPU disconnectedness
        self.score().cmp(&other.score())
    }
}

impl PartialOrd for DeviceKind {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for DeviceKind {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DeviceKind::*;
        match self {
            Discrete   => write!(f, "Discrete GPU"),
            Integrated => write!(f, "Integrated GPU"),
            Virtual    => write!(f, "Virtual GPU"),
            Cpu        => write!(f, "CPU"),
            Other      => write!(f, "Other"),
        }
    }
}

enum_from!(impl From<vk::PhysicalDeviceType> for DeviceKind {
    vk::PhysicalDeviceType::DISCRETE_GPU   => DeviceKind::Discrete,
    vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceKind::Integrated,
    vk::PhysicalDeviceType::VIRTUAL_GPU    => DeviceKind::Virtual,
    vk::PhysicalDeviceType::CPU            => DeviceKind::Cpu,
                                           => DeviceKind::Other,
});



/// An enum that describes device extensions used in the Game.
#[derive(Copy, Clone, Debug)]
pub enum DeviceExtension {
    /// The Swapchain device extension.
    Swapchain,
    /// The portability subset extension.
    PortabilitySubset,
    /// The 8-bit index extension.
    SmallIndices,
}

impl DeviceExtension {
    /// Constant function to get the string value of the DeviceExtension.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        use DeviceExtension::*;
        match self {
            Swapchain         => "VK_KHR_swapchain",
            PortabilitySubset => "VK_KHR_portability_subset",
            SmallIndices      => "VK_EXT_index_type_uint8",
        }
    }
}

impl Display for DeviceExtension {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.as_str())
    }
}

impl From<DeviceExtension> for CString {
    #[inline]
    fn from(value: DeviceExtension) -> Self {
        to_cstring!(format!("{}", value))
    }
}

impl FromStr for DeviceExtension {
    type Err = ExtensionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "VK_KHR_swapchain"          => Ok(DeviceExtension::Swapchain),
            "VK_KHR_portability_subset" => Ok(DeviceExtension::PortabilitySubset),
            "VK_EXT_index_type_uint8"   => Ok(DeviceExtension::SmallIndices),
            value                       => Err(ExtensionError::UnknownDeviceExtension{ got: value.into() }),
        }
    }
}



/// An enum that describes device layers used in the Game.
#[derive(Copy, Clone, Debug)]
pub enum DeviceLayer {
    /// A dummy extension as a temporary placeholder
    Dummy,
}

impl DeviceLayer {
    /// Constant function to get the string value of the DeviceLayer.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        use DeviceLayer::*;
        match self {
            Dummy => "dummy",
        }
    }
}

impl Display for DeviceLayer {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DeviceLayer {
    type Err = ExtensionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "dummy" => Ok(DeviceLayer::Dummy),
            value   => Err(ExtensionError::UnknownDeviceLayer{ got: value.into() }),
        }
    }
}





/***** QUEUES *****/
/// Enum that defines the types of queues that the Game has.
#[derive(Clone, Copy, Debug)]
pub enum QueueKind {
    /// The queue that is used for graphics operations (rendering & (technically) presenting)
    Graphics,
    /// The queue that is used for memory operations (transferring)
    Memory,
    /// The queue that is used for present operations (& technically rendering)
    Present,
    /// The queue that is used for compute operations
    Compute,
}





/***** DESCRIPTOR SETS / LAYOUTS *****/
/// Defines the possible Descriptor types.
#[derive(Clone, Copy, Debug)]
pub enum DescriptorKind {
    /// Describes a uniform buffer.
    UniformBuffer,
    /// Describes a storage buffer.
    StorageBuffer, 
    /// Describes a dynamic uniform buffer.
    UniformDynamicBuffer,
    /// Describes a dynamic storage buffer.
    StorageDynamicBuffer, 
    /// Describes a uniform texel buffer.
    UniformTexelBuffer,
    /// Describes a storage texel buffer.
    StorageTexelBuffer, 

    /// Describes an input attachment.
    InputAttachment,
    /// Describes a single storage image.
    StorageImage,
    /// Describes a single, sampled image.
    SampledImage,

    /// Describes a texture sampler.
    Sampler,
    /// Describes a combined image sampler.
    CombindImageSampler,
}

enum_from!(impl From<vk::DescriptorType> for DescriptorKind {
    vk::DescriptorType::UNIFORM_BUFFER         => DescriptorKind::UniformBuffer,
    vk::DescriptorType::STORAGE_BUFFER         => DescriptorKind::StorageBuffer,
    vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC => DescriptorKind::UniformDynamicBuffer,
    vk::DescriptorType::STORAGE_BUFFER_DYNAMIC => DescriptorKind::StorageDynamicBuffer,
    vk::DescriptorType::UNIFORM_TEXEL_BUFFER   => DescriptorKind::UniformTexelBuffer,
    vk::DescriptorType::STORAGE_TEXEL_BUFFER   => DescriptorKind::StorageTexelBuffer,

    vk::DescriptorType::INPUT_ATTACHMENT => DescriptorKind::InputAttachment,
    vk::DescriptorType::STORAGE_IMAGE    => DescriptorKind::StorageImage,
    vk::DescriptorType::SAMPLED_IMAGE    => DescriptorKind::SampledImage,

    vk::DescriptorType::SAMPLER                => DescriptorKind::Sampler,
    vk::DescriptorType::COMBINED_IMAGE_SAMPLER => DescriptorKind::CombindImageSampler,
});





/***** RENDER PASSES *****/
/// Defines a load operation for attachments.
#[derive(Clone, Copy, Debug)]
pub enum AttachmentLoadOp {
    /// We don't care what the value of the attachment is (so they'll be undefined).
    /// 
    /// # Synchronization
    /// - For colour attachments, this uses the `VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT` operation (???).
    /// - For depth / stencil attachments, this uses the `VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT` operation (???).
    DontCare,

    /// Clear the attachment upon loading. The clear value is specified in the RenderPass.
    /// 
    /// # Synchronization
    /// - For colour attachments, this uses the `VK_ACCESS_COLOR_ATTACHMENT_READ_BIT` operation.
    /// - For depth / stencil attachments, this uses the `VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT` operation.
    Clear,
    /// Loads whatever values where already in the attachment.
    /// 
    /// # Synchronization
    /// - For colour attachments, this uses the `VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT` operation.
    /// - For depth / stencil attachments, this uses the `VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT` operation.
    Load,
}

enum_from!(impl From<vk::AttachmentLoadOp> for AttachmentLoadOp {
    vk::AttachmentLoadOp::DONT_CARE => AttachmentLoadOp::DontCare,

    vk::AttachmentLoadOp::CLEAR => AttachmentLoadOp::Clear,
    vk::AttachmentLoadOp::LOAD  => AttachmentLoadOp::Load,
});



/// Defines a store operation for attachments.
#[derive(Clone, Copy, Debug)]
pub enum AttachmentStoreOp {
    /// We don't care what the value of the attachment will be (so they'll be undefined).
    /// 
    /// # Synchronization
    /// - For colour attachments, this uses the `VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT` operation (???).
    /// - For depth / stencil attachments, this uses the `VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT` operation (???).
    DontCare,

    /// Stores the values of the attachment 'permanently' so they may be propagated to the next subpass / presentation.
    /// 
    /// # Synchronization
    /// - For colour attachments, this uses the `VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT` operation.
    /// - For depth / stencil attachments, this uses the `VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT` operation.
    Store,
}

enum_from!(impl From<vk::AttachmentStoreOp> for AttachmentStoreOp {
    vk::AttachmentStoreOp::DONT_CARE => AttachmentStoreOp::DontCare,
    vk::AttachmentStoreOp::STORE => AttachmentStoreOp::Store,
});



/// The point where a subpass will be attached to the pipeline.
#[derive(Clone, Copy, Debug)]
pub enum BindPoint {
    /// The subpass will be attached in the graphics-part of the pipeline.
    Graphics,
    /// The subpass will be attached in the compute-part of the pipeline.
    Compute,
}

enum_from!(impl From<vk::PipelineBindPoint> for BindPoint {
    vk::PipelineBindPoint::GRAPHICS => BindPoint::Graphics,
    vk::PipelineBindPoint::COMPUTE  => BindPoint::Compute,
});





/***** PIPELINE *****/
/// Defines the possible layouts for an attribute
#[derive(Clone, Copy, Debug)]
pub enum AttributeLayout {
    /// A two-dimensional vector of 32-bit floating-point numbers
    Float2,
    /// A three-dimensional vector of 32-bit floating-point numbers
    Float3,
}

impl TryFrom<vk::Format> for AttributeLayout {
    type Error = AttributeLayoutError;

    fn try_from(value: vk::Format) -> Result<Self, Self::Error> {
        match value {
            vk::Format::R32G32_SFLOAT    => Ok(AttributeLayout::Float2),
            vk::Format::R32G32B32_SFLOAT => Ok(AttributeLayout::Float3),
            value                        => Err(AttributeLayoutError::IllegalFormatValue{ value }),
        }
    }
}

impl From<AttributeLayout> for vk::Format {
    fn from(value: AttributeLayout) -> Self {
        match value {
            AttributeLayout::Float2 => vk::Format::R32G32_SFLOAT,
            AttributeLayout::Float3 => vk::Format::R32G32B32_SFLOAT,
        }
    }
}



/// Defines how vertices will be read from the buffer (specifically, direct or instanced)
#[derive(Clone, Copy, Debug)]
pub enum VertexInputRate {
    /// Input the vertices as-is
    Vertex,
    /// Render instance-based
    Instance,
}

enum_from!(impl From<vk::VertexInputRate> for VertexInputRate {
    vk::VertexInputRate::VERTEX   => VertexInputRate::Vertex,
    vk::VertexInputRate::INSTANCE => VertexInputRate::Instance,
});



/// Defines the possible topologies for input vertices.
#[derive(Clone, Copy, Debug)]
pub enum VertexTopology {
    /// The input vertices each define separate points
    PointList,

    /// The input vertices define a list of separate lines.
    /// 
    /// Concretely, every consecutive set of two vertices define a line.
    LineList,
    /// The input vertices define a list of consecutive lines.
    /// 
    /// Concretely, the first consecutive set of two vertices defines a line. Then, every consecutive new vertex defines a line with the previous vertex.
    LineStrip,
    /// The input vertices define a list of separate lines with adjacent points.
    /// 
    /// Concretely, every consecutive set of four vertices define a line, drawn between the second and third vertex. The other two are not drawn, but only accessible in the geometry shader.
    LineListAdjacency,
    /// The input vertices define a list of consecutive lines with adjacent points.
    /// 
    /// Concretely, the very first vertex is skipped. The subsequent consecutive set of two vertices defines a line. Then, every consecutive new vertex defines a line with the previous vertex, except for the last vertex. That and the first vertex are only accessible in the geometry shader.
    LineStripAdjacency,

    /// The input vertices define a list of separate triangles.
    /// 
    /// Concretely, every consecutive set of three vertices define a triangle.
    TriangleList,
    /// The input vertices define a list of triangles that share edges.
    /// 
    /// Concretely, the first consecutive set of three vertices defines a triangle. Then, every consecutive new vertex defines a triangle with the previous two vertices.
    TriangleStrip,
    /// The input vertices define a list of triangles that share a common origin vertex.
    /// 
    /// Concretely, the first consecutive set of three vertices defines a triangle. Then, every consecutive set of two vertices defines a triangle with the first vertex in the list.
    /// 
    /// Note that this mode might do funky with some sort of portability mode (see <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#drawing-triangle-fans>)
    TriangleFan,
    /// The input vertices define a list of separate triangles with adjacent points.
    /// 
    /// Concretely, every consecutive set of six vertices define a triangle, drawn between the first, third and fifth vertex. The other three are not drawn, but only accessible in the geometry shader.
    TriangleListAdjacency,
    /// The input vertices define a list of triangles that share edges.
    /// 
    /// Concretely, the first consecutive set of five vertices defines a triangle, drawn between the first, third and fifth vertex. Then, every consecutive set of two vertices defines a triangle with the the second of those vertices and the previous two (drawn) vertices. The other vertices are not drawn, but only accessible in the geometry shader.
    TriangleStripAdjacency,

    /// The input vertices define no particular shape.
    /// 
    /// Concretely, the vertices are treated to belong to the same shape, and will not be send to vertex post-processing. Instead, they should be used in tessellation to generate renderable primitives.
    PatchList,
}

enum_from!(impl From<vk::PrimitiveTopology> for VertexTopology {
    vk::PrimitiveTopology::POINT_LIST => VertexTopology::PointList,

    vk::PrimitiveTopology::LINE_LIST                 => VertexTopology::LineList,
    vk::PrimitiveTopology::LINE_STRIP                => VertexTopology::LineStrip,
    vk::PrimitiveTopology::LINE_LIST_WITH_ADJACENCY  => VertexTopology::LineListAdjacency,
    vk::PrimitiveTopology::LINE_STRIP_WITH_ADJACENCY => VertexTopology::LineStripAdjacency,

    vk::PrimitiveTopology::TRIANGLE_LIST                 => VertexTopology::TriangleList,
    vk::PrimitiveTopology::TRIANGLE_STRIP                => VertexTopology::TriangleStrip,
    vk::PrimitiveTopology::TRIANGLE_FAN                  => VertexTopology::TriangleFan,
    vk::PrimitiveTopology::TRIANGLE_LIST_WITH_ADJACENCY  => VertexTopology::TriangleListAdjacency,
    vk::PrimitiveTopology::TRIANGLE_STRIP_WITH_ADJACENCY => VertexTopology::TriangleStripAdjacency,

    vk::PrimitiveTopology::PATCH_LIST => VertexTopology::PatchList,
});



/// Defines the possible culling modes (i.e., how to discard vertices based on their winding order).
#[derive(Clone, Copy, Debug)]
pub enum CullMode {
    /// Cull vertices that we see from both the front and the back (lol)
    FrontAndBack,
    /// Only cull vertices facing us
    Front,
    /// Only cull vertices facing away from us
    Back,
    /// Do not cull any vertices
    None,
}

enum_from!(impl From<vk::CullModeFlags> for CullMode {
    vk::CullModeFlags::FRONT_AND_BACK => CullMode::FrontAndBack,
    vk::CullModeFlags::FRONT          => CullMode::Front,
    vk::CullModeFlags::BACK           => CullMode::Back,
    vk::CullModeFlags::NONE           => CullMode::None,
});



/// Defines which winding direction we consider to be 'front'
#[derive(Clone, Copy, Debug)]
pub enum FrontFace {
    /// The clockwise-winded triangles are 'front'
    Clockwise,
    /// The counter-clockwise-winded triangles are 'front'
    CounterClockwise,
}

enum_from!(impl From<vk::FrontFace> for FrontFace {
    vk::FrontFace::CLOCKWISE         => FrontFace::Clockwise,
    vk::FrontFace::COUNTER_CLOCKWISE => FrontFace::CounterClockwise,
});



/// Defines how to draw in-between the vertices
#[derive(Clone, Copy, Debug)]
pub enum DrawMode {
    /// Only draw the points of the primitive shape
    Point,
    /// Only draws the countours of the primitive shape
    Line,
    /// Fills the entire shape
    Fill,
}

enum_from!(impl From<vk::PolygonMode> for DrawMode {
    vk::PolygonMode::POINT => DrawMode::Point,
    vk::PolygonMode::LINE  => DrawMode::Line,
    vk::PolygonMode::FILL  => DrawMode::Fill,
});



/// Defines possible operations for stencils.
#[derive(Clone, Copy, Debug)]
pub enum StencilOp {
    /// Keeps the fragment (or something else)
    Keep,
    /// Sets its value to 0
    Zero,
    /// Replaces the fragment with another value
    Replace,
    /// Inverts the value of the fragment bitwise
    Invert,

    /// Increments the value and clamps it to the maximum representable value
    IncrementClamp,
    /// Decrements the value and clamps it to 0
    DecrementClamp,

    /// Increments the value and wraps it around the maximum representable value back to 0
    IncrementWrap,
    /// Decrements the value and wraps it around 0 back to the maximum representable value
    DecrementWrap,
}

enum_from!(impl From<vk::StencilOp> for StencilOp {
    vk::StencilOp::KEEP    => StencilOp::Keep,
    vk::StencilOp::ZERO    => StencilOp::Zero,
    vk::StencilOp::REPLACE => StencilOp::Replace,
    vk::StencilOp::INVERT  => StencilOp::Invert,

    vk::StencilOp::INCREMENT_AND_CLAMP => StencilOp::IncrementClamp,
    vk::StencilOp::DECREMENT_AND_CLAMP => StencilOp::DecrementClamp,

    vk::StencilOp::INCREMENT_AND_WRAP => StencilOp::IncrementWrap,
    vk::StencilOp::DECREMENT_AND_WRAP => StencilOp::DecrementWrap,
});



/// Defines possible comparison operations.
#[derive(Clone, Copy, Debug)]
pub enum CompareOp {
    /// The comparison always succeeds
    Always,
    /// The comparison never succeeds (always fails)
    Never,

    /// The comparison succeeds iff A < B
    Less,
    /// The comparison succeeds iff A <= B
    LessEq,
    /// The comparison succeeds iff A > B
    Greater,
    /// The comparison succeeds iff A >= B
    GreaterEq,
    /// The comparison succeeds iff A == B
    Equal,
    /// The comparison succeeds iff A != B
    NotEqual,
}

enum_from!(impl From<vk::CompareOp> for CompareOp {
    vk::CompareOp::ALWAYS => CompareOp::Always,
    vk::CompareOp::NEVER  => CompareOp::Never,

    vk::CompareOp::LESS             => CompareOp::Less,
    vk::CompareOp::LESS_OR_EQUAL    => CompareOp::LessEq,
    vk::CompareOp::GREATER          => CompareOp::Greater,
    vk::CompareOp::GREATER_OR_EQUAL => CompareOp::GreaterEq,
    vk::CompareOp::EQUAL            => CompareOp::Equal,
    vk::CompareOp::NOT_EQUAL        => CompareOp::NotEqual,
});



/// Defines logic operations to perform.
#[derive(Clone, Copy, Debug)]
pub enum LogicOp {
    /// Leaves the destination as-is (`d = d`)
    NoOp,
    /// Set the bits of the destination to 0 (`d = 0`)
    Clear,
    /// Set the bits of the destination to 1 (`d = ~0`)
    Set,
    /// Copies the bits of the source to the destination (`d = s`)
    Copy,
    /// Copies the bits of the source after negating them (`d = ~s`)
    CopyInv,

    /// Negates the destination (`d = ~d`)
    Not,

    /// Performs a bitwise-AND (`d = s & d`)
    And,
    /// Performs a bitwise-AND, negating the source (`d = ~s & d`)
    AndInv,
    /// Performs a bitwise-AND, negating the destination (`d = s & ~d`)
    AndRev,
    /// Performs a negated bitwise-AND (`d = ~(s & d)`)
    NAnd,

    /// Performs a bitwise-XOR (`d = s ^ d`)
    Xor,
    /// Performs a negated bitwise-XOR (`d = ~(s ^ d)`)
    NXor,

    /// Performs a bitwise-OR (`d = s | d`)
    Or,
    /// Performs a bitwise-OR, negating the source (`d = ~s | d`)
    OrInv,
    /// Performs a bitwise-OR, negating the destination (`d = s | ~d`)
    OrRev,
    /// Performs a negated bitwise-OR (`d = ~(s | d)`)
    NOr,
}

enum_from!(impl From<vk::LogicOp> for LogicOp {
    vk::LogicOp::NO_OP         => LogicOp::NoOp,
    vk::LogicOp::CLEAR         => LogicOp::Clear,
    vk::LogicOp::SET           => LogicOp::Set,
    vk::LogicOp::COPY          => LogicOp::Copy,
    vk::LogicOp::COPY_INVERTED => LogicOp::CopyInv,

    vk::LogicOp::INVERT => LogicOp::Not,

    vk::LogicOp::AND          => LogicOp::And,
    vk::LogicOp::AND_INVERTED => LogicOp::AndInv,
    vk::LogicOp::AND_REVERSE  => LogicOp::AndRev,
    vk::LogicOp::NAND         => LogicOp::NAnd,

    vk::LogicOp::XOR        => LogicOp::Xor,
    vk::LogicOp::EQUIVALENT => LogicOp::NXor,

    vk::LogicOp::OR          => LogicOp::Or,
    vk::LogicOp::OR_INVERTED => LogicOp::OrInv,
    vk::LogicOp::OR_REVERSE  => LogicOp::OrRev,
    vk::LogicOp::NOR         => LogicOp::NOr,
});



/// Defines the factor of some value to take in a blending operation.
#[derive(Clone, Copy, Debug)]
pub enum BlendFactor {
    /// Use none of the colour (`(0.0, 0.0, 0.0, 0.0)`)
    Zero,
    /// Use all of the colour (`(1.0, 1.0, 1.0, 1.0)`)
    One,

    /// Use the source colour as the factor in blending (`(Rs, Gs, Bs, As)`)
    SrcColour,
    /// Use one minus the source colour as the factor in blending (`(1.0 - Rs, 1.0 - Gs, 1.0 - Bs, 1.0 - As)`)
    OneMinusSrcColour,
    /// Use the destination colour as the factor in blending (`(Rd, Gd, Bd, Ad)`)
    DstColour,
    /// Use one minus the destination colour as the factor in blending (`(1.0 - Rd, 1.0 - Gd, 1.0 - Bd, 1.0 - Ad)`)
    OneMinusDstColour,

    /// Use the source alpha as the factor in blending (`(As, As, As, As)`)
    SrcAlpha,
    /// Use one minus the source alpha as the factor in blending (`(1.0 - As, 1.0 - As, 1.0 - As, 1.0 - As)`)
    OneMinusSrcAlpha,
    /// Use the destination alpha as the factor in blending (`(Ad, Ad, Ad, Ad)`)
    DstAlpha,
    /// Use one minus the destination alpha as the factor in blending (`(1.0 - Ad, 1.0 - Ad, 1.0 - Ad, 1.0 - Ad)`)
    OneMinusDstAlpha,

    /// Use the constant factors given in the ColourBlendState as the factors (`(Fr, Fg, Fb, Fa)`)
    ConstColour,
    /// Use one minus the constant factors given in the ColourBlendState as the factors (`(1.0 - Fr, 1.0 - Fg, 1.0 - Fb, 1.0 - Fa)`)
    OneMinusConstColour,
    /// Use the constant alpha factor given in the ColourBlendState as the factors (`(Fa, Fa, Fa, Fa)`)
    ConstAlpha,
    /// Use one minus the constant alpha factor given in the ColourBlendState as the factors (`(1.0 - Fa, 1.0 - Fa, 1.0 - Fa, 1.0 - Fa)`)
    OneMinusConstAlpha,

    /// When using double source channels, use the colour of the second channel (`(Rs2, Gs2, Bs2, As2)`).
    SrcColour2,
    /// When using double source channels, use one minus the colour of the second channel (`(1.0 - Rs2, 1.0 - Gs2, 1.0 - Bs2, 1.0 - As2)`).
    OneMinusSrcColour2,
    /// When using double source channels, use the alpha of the second channel (`(As2, As2, As2, As2)`).
    SrcAlpha2,
    /// When using double source channels, use one minus the alpha of the second channel (`(1.0 - As2, 1.0 - As2, 1.0 - As2, 1.0 - As2)`).
    OneMinusSrcAlpha2,

    /// Saturates the colour according to the alpha channel (`(min(As, 1.0 - Ad), min(As, 1.0 - Ad), min(As, 1.0 - Ad), 1.0)`)
    SrcAlphaSaturate,
}

enum_from!(impl From<vk::BlendFactor> for BlendFactor {
    vk::BlendFactor::ZERO => BlendFactor::Zero,
    vk::BlendFactor::ONE  => BlendFactor::One,

    vk::BlendFactor::SRC_COLOR           => BlendFactor::SrcColour,
    vk::BlendFactor::ONE_MINUS_SRC_COLOR => BlendFactor::OneMinusSrcColour,
    vk::BlendFactor::DST_COLOR           => BlendFactor::DstColour,
    vk::BlendFactor::ONE_MINUS_DST_COLOR => BlendFactor::OneMinusDstColour,

    vk::BlendFactor::SRC_ALPHA           => BlendFactor::SrcAlpha,
    vk::BlendFactor::ONE_MINUS_SRC_ALPHA => BlendFactor::OneMinusSrcAlpha,
    vk::BlendFactor::DST_ALPHA           => BlendFactor::DstAlpha,
    vk::BlendFactor::ONE_MINUS_DST_ALPHA => BlendFactor::OneMinusDstAlpha,

    vk::BlendFactor::CONSTANT_COLOR           => BlendFactor::ConstColour,
    vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR => BlendFactor::OneMinusConstColour,
    vk::BlendFactor::CONSTANT_ALPHA           => BlendFactor::ConstAlpha,
    vk::BlendFactor::ONE_MINUS_CONSTANT_ALPHA => BlendFactor::OneMinusConstAlpha,

    vk::BlendFactor::SRC1_COLOR           => BlendFactor::SrcColour2,
    vk::BlendFactor::ONE_MINUS_SRC1_COLOR => BlendFactor::OneMinusSrcColour2,
    vk::BlendFactor::SRC1_ALPHA           => BlendFactor::SrcAlpha2,
    vk::BlendFactor::ONE_MINUS_SRC1_ALPHA => BlendFactor::OneMinusSrcAlpha2,

    vk::BlendFactor::SRC_ALPHA_SATURATE => BlendFactor::SrcAlphaSaturate,
});



/// Defines blend operations to perform.
#[derive(Clone, Copy, Debug)]
pub enum BlendOp {
    /// Add the proper fractions of the colours together:
    /// ```math
    /// Rd = Rs * FCs + Rd * FCd
    /// Gd = Gs * FCs + Gd * FCd
    /// Bd = Bs * FCs + Bd * FCd
    /// Ad = As * FAs + Ad * FAd
    /// ```
    /// (`Xs` is the source channel, `Xd` is the destination channel, `FCx` is the source or destination colour fraction and `FAx` is the source or destination alpha fraction)
    Add,
    /// Subtract the proper fractions of the colours from each other:
    /// ```math
    /// Rd = Rs * FCs - Rd * FCd
    /// Gd = Gs * FCs - Gd * FCd
    /// Bd = Bs * FCs - Bd * FCd
    /// Ad = As * FAs - Ad * FAd
    /// ```
    /// (`Xs` is the source channel, `Xd` is the destination channel, `FCx` is the source or destination colour fraction and `FAx` is the source or destination alpha fraction)
    Sub,
    /// Subtract the proper fractions of the colours from each other:
    /// ```math
    /// Rd = Rd * FCd - Rs * FCs
    /// Gd = Gd * FCd - Gs * FCs
    /// Bd = Bd * FCd - Bs * FCs
    /// Ad = Ad * FAd - As * FAs
    /// ```
    /// (`Xs` is the source channel, `Xd` is the destination channel, `FCx` is the source or destination colour fraction and `FAx` is the source or destination alpha fraction)
    SubRev,

    /// Take the minimal value of the colours (ignoring fractions):
    /// ```math
    /// Rd = min(Rs, Rd)
    /// Gd = min(Gs, Gd)
    /// Bd = min(Bs, Bd)
    /// Ad = min(As, Ad)
    /// ```
    /// (`Xs` is the source channel and `Xd` is the destination channel)
    Min,
    /// Take the maximum value of the colours (ignoring fractions):
    /// ```math
    /// Rd = max(Rs, Rd)
    /// Gd = max(Gs, Gd)
    /// Bd = max(Bs, Bd)
    /// Ad = max(As, Ad)
    /// ```
    /// (`Xs` is the source channel and `Xd` is the destination channel)
    Max,
}

enum_from!(impl From<vk::BlendOp> for BlendOp {
    vk::BlendOp::ADD              => BlendOp::Add,
    vk::BlendOp::SUBTRACT         => BlendOp::Sub,
    vk::BlendOp::REVERSE_SUBTRACT => BlendOp::SubRev,

    vk::BlendOp::MIN => BlendOp::Min,
    vk::BlendOp::MAX => BlendOp::Max,
});



/// Determines whether certain states of the pipeline may later be overridden.
#[derive(Clone, Copy, Debug)]
pub enum DynamicState {
    /// The output viewport is dynamic.
    Viewport,
    /// The output scissor is dynamic.
    Scissor,
    /// The render line width is dynamic.
    LineWidth,
    /// The depth bias is dynamic.
    DepthBias,
    /// Depth bounds are dynamic.
    DepthBounds,
    /// Blend constants are dynamic.
    BlendConstants,
    /// Stencil compare masks are dynamic.
    StencilCompareMask,
    /// Stencil write masks are dynamic.
    StencilWriteMask,
    /// Stencil references are dynamic.
    StencilReference,
}

enum_from!(impl From<vk::DynamicState> for DynamicState {
    vk::DynamicState::VIEWPORT             => DynamicState::Viewport,
    vk::DynamicState::SCISSOR              => DynamicState::Scissor,
    vk::DynamicState::LINE_WIDTH           => DynamicState::LineWidth,
    vk::DynamicState::DEPTH_BIAS           => DynamicState::DepthBias,
    vk::DynamicState::DEPTH_BOUNDS         => DynamicState::DepthBounds,
    vk::DynamicState::BLEND_CONSTANTS      => DynamicState::BlendConstants,
    vk::DynamicState::STENCIL_COMPARE_MASK => DynamicState::StencilCompareMask,
    vk::DynamicState::STENCIL_WRITE_MASK   => DynamicState::StencilWriteMask,
    vk::DynamicState::STENCIL_REFERENCE    => DynamicState::StencilReference,
});





/***** COMMAND POOLS *****/
/// Possible levels for a CommandBuffer.
#[derive(Clone, Copy, Debug)]
pub enum CommandBufferLevel {
    /// The command buffer is primary, i.e., only able to be submitted to a queue.
    Primary,
    /// The command buffer is secondary, i.e., only able to be called from another (primary) command buffer.
    Secondary,
}

enum_from!(impl From<vk::CommandBufferLevel> for CommandBufferLevel {
    vk::CommandBufferLevel::PRIMARY   => CommandBufferLevel::Primary,
    vk::CommandBufferLevel::SECONDARY => CommandBufferLevel::Secondary,
});





/***** MEMORY POOLS *****/
/// Determines how a Buffer may be accessed.
#[derive(Clone, Debug)]
pub enum SharingMode {
    /// The buffer may be accessed by one queue family only. First come, first serve.
    Exclusive,
    /// The buffer may be accessed by multiple queue families. The queues have to be specified, though, as a list of queue family indices.
    Concurrent(Vec<u32>),
}

impl SharingMode {
    /// Construct the SharingMode from a given VkSharingMode and a (possible) list of concurrent queue family indices.
    #[inline]
    pub fn from_vk(sharing_mode: vk::SharingMode, queue_family_indices: Option<&[u32]>) -> Self {
        // Simply match the sharing mode
        match sharing_mode {
            vk::SharingMode::EXCLUSIVE  => SharingMode::Exclusive,
            vk::SharingMode::CONCURRENT => SharingMode::Concurrent(queue_family_indices.expect("Cannot set SharingMode to SharingMode::Concurrent without specifying a list of allowed queue family indices").into()),
            sharing_mode                => { panic!("Encountered illegal VkSharingMode value '{}'", sharing_mode.as_raw()); }
        }
    }
}

impl From<SharingMode> for (vk::SharingMode, Option<Vec<u32>>) {
    #[inline]
    fn from(value: SharingMode) -> Self {
        match value {
            SharingMode::Exclusive           => (vk::SharingMode::EXCLUSIVE, None),
            SharingMode::Concurrent(indices) => (vk::SharingMode::CONCURRENT, Some(indices)),
        }
    }
}



/// Determines the type of indices in an IndexBuffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexType {
    /// An 8-bit index type (i.e., unsigned 8-bit integer).
    /// 
    /// Note that for this IndexType, an extension is needed (VK_EXT_index_type_uint8)
    UInt8,
    /// A 16-bit index type (i.e., unsigned 16-bit integer).
    UInt16,
    /// A 32-bit index type (i.e., unsigned 32-bit integer).
    UInt32,
}

impl IndexType {
    /// Returns the size (in bytes) of this IndexType.
    #[inline]
    pub fn vk_size(&self) -> usize {
        match self {
            IndexType::UInt8  => 1,
            IndexType::UInt16 => 2,
            IndexType::UInt32 => 4,
        }
    }
}

enum_from!(impl From<vk::IndexType> for IndexType {
    vk::IndexType::UINT8_EXT => IndexType::UInt8,
    vk::IndexType::UINT16    => IndexType::UInt16,
    vk::IndexType::UINT32    => IndexType::UInt32,
});



/// Determines the kind of memory allocator supported by the MemoryPool.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAllocatorKind {
    /// Defines a "normal" allocator, that is reasonably space efficient but not so much time-wise.
    Dense,
    /// Defines a linear allocator, which is fast in allocation but which will not re-use deallocated buffer space.
    /// 
    /// The index in this linear allocator is referencing some block that was allocated beforehand.
    Linear(u64),
}

impl Default for MemoryAllocatorKind {
    fn default() -> Self { MemoryAllocatorKind::Dense }
}

impl Display for MemoryAllocatorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use MemoryAllocatorKind::*;
        match self {
            Dense      => write!(f, "Dense"),
            Linear(id) => write!(f, "Linear({})", id),
        }
    }
}





/***** IMAGES *****/
/// Defines how to re-map components.
#[derive(Debug, Clone)]
pub enum ComponentSwizzle {
    /// Do not swizzle anything
    Identity,
    /// Sets the target component to '1'.
    One,
    /// Sets the target component to '0'.
    Zero,

    /// The target component will get the value of the red channel.
    Red,
    /// The target component will get the value of the green channel.
    Green,
    /// The target component will get the value of the blue channel.
    Blue,
    /// The target component will get the value of the alpha channel.
    Alpha,
}

enum_from!(impl From<vk::ComponentSwizzle> for ComponentSwizzle {
    vk::ComponentSwizzle::IDENTITY => ComponentSwizzle::Identity,
    vk::ComponentSwizzle::ONE      => ComponentSwizzle::One,
    vk::ComponentSwizzle::ZERO     => ComponentSwizzle::Zero,
    vk::ComponentSwizzle::R        => ComponentSwizzle::Red,
    vk::ComponentSwizzle::B        => ComponentSwizzle::Blue,
    vk::ComponentSwizzle::G        => ComponentSwizzle::Green,
    vk::ComponentSwizzle::A        => ComponentSwizzle::Alpha,
});



/// The type of the ImageView
#[derive(Clone, Copy, Debug)]
pub enum ImageViewKind {
    /// A simple, one-dimensional image (i.e., a line of pixels)
    OneD,
    /// A simple, one-dimensional image but as an array (i.e., for stereophonic 3D)
    OneDArray,

    /// A simple, two-dimensional image (i.e., a grid of pixels)
    TwoD,
    /// A simple, two-dimensional image but as an array (i.e., for stereophonic 3D)
    TwoDArray,

    /// A simple, three-dimensional image
    ThreeD,

    /// A cubic (3D?) image
    Cube,
    /// A cubic (3D?) image but an array (i.e., for stereophonic 3D)
    CubeArray,
}

impl Default for ImageViewKind {
    #[inline]
    fn default() -> Self {
        ImageViewKind::TwoD
    }
}

impl Display for ImageViewKind {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ImageViewKind::*;
        match self {
            OneD      => write!(f, "1D"),
            OneDArray => write!(f, "1D (Array)"),
            TwoD      => write!(f, "2D"),
            TwoDArray => write!(f, "2D (Array)"),
            ThreeD    => write!(f, "3D"),
            Cube      => write!(f, "Cube"),
            CubeArray => write!(f, "Cube (Array)"),
        }
    }
}

enum_from!(impl From<vk::ImageViewType> for ImageViewKind {
    vk::ImageViewType::TYPE_1D       => ImageViewKind::OneD,
    vk::ImageViewType::TYPE_1D_ARRAY => ImageViewKind::OneDArray,
    vk::ImageViewType::TYPE_2D       => ImageViewKind::TwoD,
    vk::ImageViewType::TYPE_2D_ARRAY => ImageViewKind::TwoDArray,
    vk::ImageViewType::TYPE_3D       => ImageViewKind::ThreeD,
    vk::ImageViewType::CUBE          => ImageViewKind::Cube,
    vk::ImageViewType::CUBE_ARRAY    => ImageViewKind::CubeArray,
});



/// The format of an Image.
#[derive(Clone, Copy, Debug)]
pub enum ImageFormat {
    /// The format is unknown
    Undefined,

    /// R4G4_UNORM_PACK8
    R4G4UNormPack8,
    /// R4G4B4A4_UNORM_PACK16
    R4G4B4A4UNormPack16,
    /// B4G4R4A4_UNORM_PACK16
    B4G4R4A4UNormPack16,
    /// R5G6B5_UNORM_PACK16
    R5G6B5UNormPack16,
    /// B5G6R5_UNORM_PACK16
    B5G6R5UNormPack16,
    /// R5G5B5A1_UNORM_PACK16
    R5G5B5A1UNormPack16,
    /// B5G5R5A1_UNORM_PACK16
    B5G5R5A1UNormPack16,
    /// A1R5G5B5_UNORM_PACK16
    A1R5G5B5UNormPack16,
    /// R8_UNORM
    R8UNorm,
    /// R8_SNORM
    R8SNorm,
    /// R8_USCALED
    R8UScaled,
    /// R8_SSCALED
    R8SScaled,
    /// R8_UINT
    R8UInt,
    /// R8_SINT
    R8SInt,
    /// R8_SRGB
    R8SRgb,
    /// R8G8_UNORM
    R8G8UNorm,
    /// R8G8_SNORM
    R8G8SNorm,
    /// R8G8_USCALED
    R8G8UScaled,
    /// R8G8_SSCALED
    R8G8SScaled,
    /// R8G8_UINT
    R8G8UInt,
    /// R8G8_SINT
    R8G8SInt,
    /// R8G8_SRGB
    R8G8SRgb,
    /// R8G8B8_UNORM
    R8G8B8UNorm,
    /// R8G8B8_SNORM
    R8G8B8SNorm,
    /// R8G8B8_USCALED
    R8G8B8UScaled,
    /// R8G8B8_SSCALED
    R8G8B8SScaled,
    /// R8G8B8_UINT
    R8G8B8UInt,
    /// R8G8B8_SINT
    R8G8B8SInt,
    /// R8G8B8_SRGB
    R8G8B8SRgb,
    /// B8G8R8_UNORM
    B8G8R8UNorm,
    /// B8G8R8_SNORM
    B8G8R8SNorm,
    /// B8G8R8_USCALED
    B8G8R8UScaled,
    /// B8G8R8_SSCALED
    B8G8R8SScaled,
    /// B8G8R8_UINT
    B8G8R8UInt,
    /// B8G8R8_SINT
    B8G8R8SInt,
    /// B8G8R8_SRGB
    B8G8R8SRgb,
    /// R8G8B8A8_UNORM
    R8G8B8A8UNorm,
    /// R8G8B8A8_SNORM
    R8G8B8A8SNorm,
    /// R8G8B8A8_USCALED
    R8G8B8A8UScaled,
    /// R8G8B8A8_SSCALED
    R8G8B8A8SScaled,
    /// R8G8B8A8_UINT
    R8G8B8A8UInt,
    /// R8G8B8A8_SINT
    R8G8B8A8SInt,
    /// R8G8B8A8_SRGB
    R8G8B8A8SRgb,
    /// B8G8R8A8_UNORM
    B8G8R8A8UNorm,
    /// B8G8R8A8_SNORM
    B8G8R8A8SNorm,
    /// B8G8R8A8_USCALED
    B8G8R8A8UScaled,
    /// B8G8R8A8_SSCALED
    B8G8R8A8SScaled,
    /// B8G8R8A8_UINT
    B8G8R8A8UInt,
    /// B8G8R8A8_SINT
    B8G8R8A8SInt,
    /// B8G8R8A8_SRGB
    B8G8R8A8SRgb,
    /// A8B8G8R8_UNORM_PACK32
    A8B8G8R8UNormPack32,
    /// A8B8G8R8_SNORM_PACK32
    A8B8G8R8SNormPack32,
    /// A8B8G8R8_USCALED_PACK32
    A8B8G8R8UScaledPack32,
    /// A8B8G8R8_SSCALED_PACK32
    A8B8G8R8SScaledPack32,
    /// A8B8G8R8_UINT_PACK32
    A8B8G8R8UIntPack32,
    /// A8B8G8R8_SINT_PACK32
    A8B8G8R8SIntPack32,
    /// A8B8G8R8_SRGB_PACK32
    A8B8G8R8SRgbPack32,
    /// A2R10G10B10_UNORM_PACK32
    A2R10G10B10UNormPack32,
    /// A2R10G10B10_SNORM_PACK32
    A2R10G10B10SNormPack32,
    /// A2R10G10B10_USCALED_PACK32
    A2R10G10B10UScaledPack32,
    /// A2R10G10B10_SSCALED_PACK32
    A2R10G10B10SScaledPack32,
    /// A2R10G10B10_UINT_PACK32
    A2R10G10B10UIntPack32,
    /// A2R10G10B10_SINT_PACK32
    A2R10G10B10SIntPack32,
    /// A2B10G10R10_UNORM_PACK32
    A2B10G10R10UNormPack32,
    /// A2B10G10R10_SNORM_PACK32
    A2B10G10R10SNormPack32,
    /// A2B10G10R10_USCALED_PACK32
    A2B10G10R10UScaledPack32,
    /// A2B10G10R10_SSCALED_PACK32
    A2B10G10R10SScaledPack32,
    /// A2B10G10R10_UINT_PACK32
    A2B10G10R10UIntPack32,
    /// A2B10G10R10_SINT_PACK32
    A2B10G10R10SIntPack32,
    /// R16_UNORM
    R16UNorm,
    /// R16_SNORM
    R16SNorm,
    /// R16_USCALED
    R16UScaled,
    /// R16_SSCALED
    R16SScaled,
    /// R16_UINT
    R16UInt,
    /// R16_SINT
    R16SInt,
    /// R16_SFLOAT
    R16SFloat,
    /// R16G16_UNORM
    R16G16UNorm,
    /// R16G16_SNORM
    R16G16SNorm,
    /// R16G16_USCALED
    R16G16UScaled,
    /// R16G16_SSCALED
    R16G16SScaled,
    /// R16G16_UINT
    R16G16UInt,
    /// R16G16_SINT
    R16G16SInt,
    /// R16G16_SFLOAT
    R16G16SFloat,
    /// R16G16B16_UNORM
    R16G16B16UNorm,
    /// R16G16B16_SNORM
    R16G16B16SNorm,
    /// R16G16B16_USCALED
    R16G16B16UScaled,
    /// R16G16B16_SSCALED
    R16G16B16SScaled,
    /// R16G16B16_UINT
    R16G16B16UInt,
    /// R16G16B16_SINT
    R16G16B16SInt,
    /// R16G16B16_SFLOAT
    R16G16B16SFloat,
    /// R16G16B16A16_UNORM
    R16G16B16A16UNorm,
    /// R16G16B16A16_SNORM
    R16G16B16A16SNorm,
    /// R16G16B16A16_USCALED
    R16G16B16A16UScaled,
    /// R16G16B16A16_SSCALED
    R16G16B16A16SScaled,
    /// R16G16B16A16_UINT
    R16G16B16A16UInt,
    /// R16G16B16A16_SINT
    R16G16B16A16SInt,
    /// R16G16B16A16_SFLOAT
    R16G16B16A16SFloat,
    /// R32_UINT
    R32UInt,
    /// R32_SINT
    R32SInt,
    /// R32_SFLOAT
    R32SFloat,
    /// R32G32_UINT
    R32G32UInt,
    /// R32G32_SINT
    R32G32SInt,
    /// R32G32_SFLOAT
    R32G32SFloat,
    /// R32G32B32_UINT
    R32G32B32UInt,
    /// R32G32B32_SINT
    R32G32B32SInt,
    /// R32G32B32_SFLOAT
    R32G32B32SFloat,
    /// R32G32B32A32_UINT
    R32G32B32A32UInt,
    /// R32G32B32A32_SINT
    R32G32B32A32SInt,
    /// R32G32B32A32_SFLOAT
    R32G32B32A32SFloat,
    /// R64_UINT
    R64UInt,
    /// R64_SINT
    R64SInt,
    /// R64_SFLOAT
    R64SFloat,
    /// R64G64_UINT
    R64G64UInt,
    /// R64G64_SINT
    R64G64SInt,
    /// R64G64_SFLOAT
    R64G64SFloat,
    /// R64G64B64_UINT
    R64G64B64UInt,
    /// R64G64B64_SINT
    R64G64B64SInt,
    /// R64G64B64_SFLOAT
    R64G64B64SFloat,
    /// R64G64B64A64_UINT
    R64G64B64A64UInt,
    /// R64G64B64A64_SINT
    R64G64B64A64SInt,
    /// R64G64B64A64_SFLOAT
    R64G64B64A64SFloat,
    /// B10G11R11_UFLOAT_PACK32
    B10G11R11UFloatPack32,
    /// E5B9G9R9_UFLOAT_PACK32
    E5B9G9R9UFloatPack32,
    /// D16_UNORM
    D16UNorm,
    /// X8_D24_UNORM_PACK32
    X8D24UNormPack32,
    /// D32_SFLOAT
    D32SFloat,
    /// S8_UINT
    S8UInt,
    /// D16_UNORM_S8_UINT
    D16UNormS8UInt,
    /// D24_UNORM_S8_UINT
    D24UNormS8UInt,
    /// D32_SFLOAT_S8_UINT
    D32SFloatS8UInt,
    /// BC1_RGB_UNORM_BLOCK
    BC1RGBUNormBlock,
    /// BC1_RGB_SRGB_BLOCK
    BC1RGBSRgbBlock,
    /// BC1_RGBA_UNORM_BLOCK
    BC1RGBAUNormBlock,
    /// BC1_RGBA_SRGB_BLOCK
    BC1RGBASRgbBlock,
    /// BC2_UNORM_BLOCK
    BC2UNormBlock,
    /// BC2_SRGB_BLOCK
    BC2SRgbBlock,
    /// BC3_UNORM_BLOCK
    BC3UNormBlock,
    /// BC3_SRGB_BLOCK
    BC3SRgbBlock,
    /// BC4_UNORM_BLOCK
    BC4UNormBlock,
    /// BC4_SNORM_BLOCK
    BC4SNormBlock,
    /// BC5_UNORM_BLOCK
    BC5UNormBlock,
    /// BC5_SNORM_BLOCK
    BC5SNormBlock,
    /// BC6H_UFLOAT_BLOCK
    BC6HUFloatBlock,
    /// BC6H_SFLOAT_BLOCK
    BC6HSFloatBlock,
    /// BC7_UNORM_BLOCK
    BC7UNormBlock,
    /// BC7_SRGB_BLOCK
    BC7SRgbBlock,
    /// ETC2_R8G8B8_UNORM_BLOCK
    ETC2R8G8B8UNormBlock,
    /// ETC2_R8G8B8_SRGB_BLOCK
    ETC2R8G8B8SRgbBlock,
    /// ETC2_R8G8B8A1_UNORM_BLOCK
    ETC2R8G8B8A1UNormBlock,
    /// ETC2_R8G8B8A1_SRGB_BLOCK
    ETC2R8G8B8A1SRgbBlock,
    /// ETC2_R8G8B8A8_UNORM_BLOCK
    ETC2R8G8B8A8UNormBlock,
    /// ETC2_R8G8B8A8_SRGB_BLOCK
    ETC2R8G8B8A8SRgbBlock,
    /// EAC_R11_UNORM_BLOCK
    EACR11UNormBlock,
    /// EAC_R11_SNORM_BLOCK
    EACR11SNormBlock,
    /// EAC_R11G11_UNORM_BLOCK
    EACR11G11UNormBlock,
    /// EAC_R11G11_SNORM_BLOCK
    EACR11G11SNormBlock,
    /// ASTC_4X4_UNORM_BLOCK
    ASTC4X4UNormBlock,
    /// ASTC_4X4_SRGB_BLOCK
    ASTC4X4SRgbBlock,
    /// ASTC_5X4_UNORM_BLOCK
    ASTC5X4UNormBlock,
    /// ASTC_5X4_SRGB_BLOCK
    ASTC5X4SRgbBlock,
    /// ASTC_5X5_UNORM_BLOCK
    ASTC5X5UNormBlock,
    /// ASTC_5X5_SRGB_BLOCK
    ASTC5X5SRgbBlock,
    /// ASTC_6X5_UNORM_BLOCK
    ASTC6X5UNormBlock,
    /// ASTC_6X5_SRGB_BLOCK
    ASTC6X5SRgbBlock,
    /// ASTC_6X6_UNORM_BLOCK
    ASTC6X6UNormBlock,
    /// ASTC_6X6_SRGB_BLOCK
    ASTC6X6SRgbBlock,
    /// ASTC_8X5_UNORM_BLOCK
    ASTC8X5UNormBlock,
    /// ASTC_8X5_SRGB_BLOCK
    ASTC8X5SRgbBlock,
    /// ASTC_8X6_UNORM_BLOCK
    ASTC8X6UNormBlock,
    /// ASTC_8X6_SRGB_BLOCK
    ASTC8X6SRgbBlock,
    /// ASTC_8X8_UNORM_BLOCK
    ASTC8X8UNormBlock,
    /// ASTC_8X8_SRGB_BLOCK
    ASTC8X8SRgbBlock,
    /// ASTC_10X5_UNORM_BLOCK
    ASTC10X5UNormBlock,
    /// ASTC_10X5_SRGB_BLOCK
    ASTC10X5SRgbBlock,
    /// ASTC_10X6_UNORM_BLOCK
    ASTC10X6UNormBlock,
    /// ASTC_10X6_SRGB_BLOCK
    ASTC10X6SRgbBlock,
    /// ASTC_10X8_UNORM_BLOCK
    ASTC10X8UNormBlock,
    /// ASTC_10X8_SRGB_BLOCK
    ASTC10X8SRgbBlock,
    /// ASTC_10X10_UNORM_BLOCK
    ASTC10X10UNormBlock,
    /// ASTC_10X10_SRGB_BLOCK
    ASTC10X10SRgbBlock,
    /// ASTC_12X10_UNORM_BLOCK
    ASTC12X10UNormBlock,
    /// ASTC_12X10_SRGB_BLOCK
    ASTC12X10SRgbBlock,
    /// ASTC_12X12_UNORM_BLOCK
    ASTC12X12UNormBlock,
    /// ASTC_12X12_SRGB_BLOCK
    ASTC12X12SRgbBlock,
}

impl Display for ImageFormat {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ImageFormat::*;
        match self {
            Undefined => write!(f, "Undefined"),

            R4G4UNormPack8 => write!(f, "R4G4UNormPack8"),
            R4G4B4A4UNormPack16 => write!(f, "R4G4B4A4UNormPack16"),
            B4G4R4A4UNormPack16 => write!(f, "B4G4R4A4UNormPack16"),
            R5G6B5UNormPack16 => write!(f, "R5G6B5UNormPack16"),
            B5G6R5UNormPack16 => write!(f, "B5G6R5UNormPack16"),
            R5G5B5A1UNormPack16 => write!(f, "R5G5B5A1UNormPack16"),
            B5G5R5A1UNormPack16 => write!(f, "B5G5R5A1UNormPack16"),
            A1R5G5B5UNormPack16 => write!(f, "A1R5G5B5UNormPack16"),
            R8UNorm => write!(f, "R8UNorm"),
            R8SNorm => write!(f, "R8SNorm"),
            R8UScaled => write!(f, "R8UScaled"),
            R8SScaled => write!(f, "R8SScaled"),
            R8UInt => write!(f, "R8UInt"),
            R8SInt => write!(f, "R8SInt"),
            R8SRgb => write!(f, "R8SRgb"),
            R8G8UNorm => write!(f, "R8G8UNorm"),
            R8G8SNorm => write!(f, "R8G8SNorm"),
            R8G8UScaled => write!(f, "R8G8UScaled"),
            R8G8SScaled => write!(f, "R8G8SScaled"),
            R8G8UInt => write!(f, "R8G8UInt"),
            R8G8SInt => write!(f, "R8G8SInt"),
            R8G8SRgb => write!(f, "R8G8SRgb"),
            R8G8B8UNorm => write!(f, "R8G8B8UNorm"),
            R8G8B8SNorm => write!(f, "R8G8B8SNorm"),
            R8G8B8UScaled => write!(f, "R8G8B8UScaled"),
            R8G8B8SScaled => write!(f, "R8G8B8SScaled"),
            R8G8B8UInt => write!(f, "R8G8B8UInt"),
            R8G8B8SInt => write!(f, "R8G8B8SInt"),
            R8G8B8SRgb => write!(f, "R8G8B8SRgb"),
            B8G8R8UNorm => write!(f, "B8G8R8UNorm"),
            B8G8R8SNorm => write!(f, "B8G8R8SNorm"),
            B8G8R8UScaled => write!(f, "B8G8R8UScaled"),
            B8G8R8SScaled => write!(f, "B8G8R8SScaled"),
            B8G8R8UInt => write!(f, "B8G8R8UInt"),
            B8G8R8SInt => write!(f, "B8G8R8SInt"),
            B8G8R8SRgb => write!(f, "B8G8R8SRgb"),
            R8G8B8A8UNorm => write!(f, "R8G8B8A8UNorm"),
            R8G8B8A8SNorm => write!(f, "R8G8B8A8SNorm"),
            R8G8B8A8UScaled => write!(f, "R8G8B8A8UScaled"),
            R8G8B8A8SScaled => write!(f, "R8G8B8A8SScaled"),
            R8G8B8A8UInt => write!(f, "R8G8B8A8UInt"),
            R8G8B8A8SInt => write!(f, "R8G8B8A8SInt"),
            R8G8B8A8SRgb => write!(f, "R8G8B8A8SRgb"),
            B8G8R8A8UNorm => write!(f, "B8G8R8A8UNorm"),
            B8G8R8A8SNorm => write!(f, "B8G8R8A8SNorm"),
            B8G8R8A8UScaled => write!(f, "B8G8R8A8UScaled"),
            B8G8R8A8SScaled => write!(f, "B8G8R8A8SScaled"),
            B8G8R8A8UInt => write!(f, "B8G8R8A8UInt"),
            B8G8R8A8SInt => write!(f, "B8G8R8A8SInt"),
            B8G8R8A8SRgb => write!(f, "B8G8R8A8SRgb"),
            A8B8G8R8UNormPack32 => write!(f, "A8B8G8R8UNormPack32"),
            A8B8G8R8SNormPack32 => write!(f, "A8B8G8R8SNormPack32"),
            A8B8G8R8UScaledPack32 => write!(f, "A8B8G8R8UScaledPack32"),
            A8B8G8R8SScaledPack32 => write!(f, "A8B8G8R8SScaledPack32"),
            A8B8G8R8UIntPack32 => write!(f, "A8B8G8R8UIntPack32"),
            A8B8G8R8SIntPack32 => write!(f, "A8B8G8R8SIntPack32"),
            A8B8G8R8SRgbPack32 => write!(f, "A8B8G8R8SRgbPack32"),
            A2R10G10B10UNormPack32 => write!(f, "A2R10G10B10UNormPack32"),
            A2R10G10B10SNormPack32 => write!(f, "A2R10G10B10SNormPack32"),
            A2R10G10B10UScaledPack32 => write!(f, "A2R10G10B10UScaledPack32"),
            A2R10G10B10SScaledPack32 => write!(f, "A2R10G10B10SScaledPack32"),
            A2R10G10B10UIntPack32 => write!(f, "A2R10G10B10UIntPack32"),
            A2R10G10B10SIntPack32 => write!(f, "A2R10G10B10SIntPack32"),
            A2B10G10R10UNormPack32 => write!(f, "A2B10G10R10UNormPack32"),
            A2B10G10R10SNormPack32 => write!(f, "A2B10G10R10SNormPack32"),
            A2B10G10R10UScaledPack32 => write!(f, "A2B10G10R10UScaledPack32"),
            A2B10G10R10SScaledPack32 => write!(f, "A2B10G10R10SScaledPack32"),
            A2B10G10R10UIntPack32 => write!(f, "A2B10G10R10UIntPack32"),
            A2B10G10R10SIntPack32 => write!(f, "A2B10G10R10SIntPack32"),
            R16UNorm => write!(f, "R16UNorm"),
            R16SNorm => write!(f, "R16SNorm"),
            R16UScaled => write!(f, "R16UScaled"),
            R16SScaled => write!(f, "R16SScaled"),
            R16UInt => write!(f, "R16UInt"),
            R16SInt => write!(f, "R16SInt"),
            R16SFloat => write!(f, "R16SFloat"),
            R16G16UNorm => write!(f, "R16G16UNorm"),
            R16G16SNorm => write!(f, "R16G16SNorm"),
            R16G16UScaled => write!(f, "R16G16UScaled"),
            R16G16SScaled => write!(f, "R16G16SScaled"),
            R16G16UInt => write!(f, "R16G16UInt"),
            R16G16SInt => write!(f, "R16G16SInt"),
            R16G16SFloat => write!(f, "R16G16SFloat"),
            R16G16B16UNorm => write!(f, "R16G16B16UNorm"),
            R16G16B16SNorm => write!(f, "R16G16B16SNorm"),
            R16G16B16UScaled => write!(f, "R16G16B16UScaled"),
            R16G16B16SScaled => write!(f, "R16G16B16SScaled"),
            R16G16B16UInt => write!(f, "R16G16B16UInt"),
            R16G16B16SInt => write!(f, "R16G16B16SInt"),
            R16G16B16SFloat => write!(f, "R16G16B16SFloat"),
            R16G16B16A16UNorm => write!(f, "R16G16B16A16UNorm"),
            R16G16B16A16SNorm => write!(f, "R16G16B16A16SNorm"),
            R16G16B16A16UScaled => write!(f, "R16G16B16A16UScaled"),
            R16G16B16A16SScaled => write!(f, "R16G16B16A16SScaled"),
            R16G16B16A16UInt => write!(f, "R16G16B16A16UInt"),
            R16G16B16A16SInt => write!(f, "R16G16B16A16SInt"),
            R16G16B16A16SFloat => write!(f, "R16G16B16A16SFloat"),
            R32UInt => write!(f, "R32UInt"),
            R32SInt => write!(f, "R32SInt"),
            R32SFloat => write!(f, "R32SFloat"),
            R32G32UInt => write!(f, "R32G32UInt"),
            R32G32SInt => write!(f, "R32G32SInt"),
            R32G32SFloat => write!(f, "R32G32SFloat"),
            R32G32B32UInt => write!(f, "R32G32B32UInt"),
            R32G32B32SInt => write!(f, "R32G32B32SInt"),
            R32G32B32SFloat => write!(f, "R32G32B32SFloat"),
            R32G32B32A32UInt => write!(f, "R32G32B32A32UInt"),
            R32G32B32A32SInt => write!(f, "R32G32B32A32SInt"),
            R32G32B32A32SFloat => write!(f, "R32G32B32A32SFloat"),
            R64UInt => write!(f, "R64UInt"),
            R64SInt => write!(f, "R64SInt"),
            R64SFloat => write!(f, "R64SFloat"),
            R64G64UInt => write!(f, "R64G64UInt"),
            R64G64SInt => write!(f, "R64G64SInt"),
            R64G64SFloat => write!(f, "R64G64SFloat"),
            R64G64B64UInt => write!(f, "R64G64B64UInt"),
            R64G64B64SInt => write!(f, "R64G64B64SInt"),
            R64G64B64SFloat => write!(f, "R64G64B64SFloat"),
            R64G64B64A64UInt => write!(f, "R64G64B64A64UInt"),
            R64G64B64A64SInt => write!(f, "R64G64B64A64SInt"),
            R64G64B64A64SFloat => write!(f, "R64G64B64A64SFloat"),
            B10G11R11UFloatPack32 => write!(f, "B10G11R11UFloatPack32"),
            E5B9G9R9UFloatPack32 => write!(f, "E5B9G9R9UFloatPack32"),
            D16UNorm => write!(f, "D16UNorm"),
            X8D24UNormPack32 => write!(f, "X8D24UNormPack32"),
            D32SFloat => write!(f, "D32SFloat"),
            S8UInt => write!(f, "S8UInt"),
            D16UNormS8UInt => write!(f, "D16UNormS8UInt"),
            D24UNormS8UInt => write!(f, "D24UNormS8UInt"),
            D32SFloatS8UInt => write!(f, "D32SFloatS8UInt"),
            BC1RGBUNormBlock => write!(f, "BC1RGBUNormBlock"),
            BC1RGBSRgbBlock => write!(f, "BC1RGBSRgbBlock"),
            BC1RGBAUNormBlock => write!(f, "BC1RGBAUNormBlock"),
            BC1RGBASRgbBlock => write!(f, "BC1RGBASRgbBlock"),
            BC2UNormBlock => write!(f, "BC2UNormBlock"),
            BC2SRgbBlock => write!(f, "BC2SRgbBlock"),
            BC3UNormBlock => write!(f, "BC3UNormBlock"),
            BC3SRgbBlock => write!(f, "BC3SRgbBlock"),
            BC4UNormBlock => write!(f, "BC4UNormBlock"),
            BC4SNormBlock => write!(f, "BC4SNormBlock"),
            BC5UNormBlock => write!(f, "BC5UNormBlock"),
            BC5SNormBlock => write!(f, "BC5SNormBlock"),
            BC6HUFloatBlock => write!(f, "BC6HUFloatBlock"),
            BC6HSFloatBlock => write!(f, "BC6HSFloatBlock"),
            BC7UNormBlock => write!(f, "BC7UNormBlock"),
            BC7SRgbBlock => write!(f, "BC7SRgbBlock"),
            ETC2R8G8B8UNormBlock => write!(f, "ETC2R8G8B8UNormBlock"),
            ETC2R8G8B8SRgbBlock => write!(f, "ETC2R8G8B8SRgbBlock"),
            ETC2R8G8B8A1UNormBlock => write!(f, "ETC2R8G8B8A1UNormBlock"),
            ETC2R8G8B8A1SRgbBlock => write!(f, "ETC2R8G8B8A1SRgbBlock"),
            ETC2R8G8B8A8UNormBlock => write!(f, "ETC2R8G8B8A8UNormBlock"),
            ETC2R8G8B8A8SRgbBlock => write!(f, "ETC2R8G8B8A8SRgbBlock"),
            EACR11UNormBlock => write!(f, "EACR11UNormBlock"),
            EACR11SNormBlock => write!(f, "EACR11SNormBlock"),
            EACR11G11UNormBlock => write!(f, "EACR11G11UNormBlock"),
            EACR11G11SNormBlock => write!(f, "EACR11G11SNormBlock"),
            ASTC4X4UNormBlock => write!(f, "ASTC4X4UNormBlock"),
            ASTC4X4SRgbBlock => write!(f, "ASTC4X4SRgbBlock"),
            ASTC5X4UNormBlock => write!(f, "ASTC5X4UNormBlock"),
            ASTC5X4SRgbBlock => write!(f, "ASTC5X4SRgbBlock"),
            ASTC5X5UNormBlock => write!(f, "ASTC5X5UNormBlock"),
            ASTC5X5SRgbBlock => write!(f, "ASTC5X5SRgbBlock"),
            ASTC6X5UNormBlock => write!(f, "ASTC6X5UNormBlock"),
            ASTC6X5SRgbBlock => write!(f, "ASTC6X5SRgbBlock"),
            ASTC6X6UNormBlock => write!(f, "ASTC6X6UNormBlock"),
            ASTC6X6SRgbBlock => write!(f, "ASTC6X6SRgbBlock"),
            ASTC8X5UNormBlock => write!(f, "ASTC8X5UNormBlock"),
            ASTC8X5SRgbBlock => write!(f, "ASTC8X5SRgbBlock"),
            ASTC8X6UNormBlock => write!(f, "ASTC8X6UNormBlock"),
            ASTC8X6SRgbBlock => write!(f, "ASTC8X6SRgbBlock"),
            ASTC8X8UNormBlock => write!(f, "ASTC8X8UNormBlock"),
            ASTC8X8SRgbBlock => write!(f, "ASTC8X8SRgbBlock"),
            ASTC10X5UNormBlock => write!(f, "ASTC10X5UNormBlock"),
            ASTC10X5SRgbBlock => write!(f, "ASTC10X5SRgbBlock"),
            ASTC10X6UNormBlock => write!(f, "ASTC10X6UNormBlock"),
            ASTC10X6SRgbBlock => write!(f, "ASTC10X6SRgbBlock"),
            ASTC10X8UNormBlock => write!(f, "ASTC10X8UNormBlock"),
            ASTC10X8SRgbBlock => write!(f, "ASTC10X8SRgbBlock"),
            ASTC10X10UNormBlock => write!(f, "ASTC10X10UNormBlock"),
            ASTC10X10SRgbBlock => write!(f, "ASTC10X10SRgbBlock"),
            ASTC12X10UNormBlock => write!(f, "ASTC12X10UNormBlock"),
            ASTC12X10SRgbBlock => write!(f, "ASTC12X10SRgbBlock"),
            ASTC12X12UNormBlock => write!(f, "ASTC12X12UNormBlock"),
            ASTC12X12SRgbBlock => write!(f, "ASTC12X12SRgbBlock"),
        }
    }
}

enum_from!(impl From<vk::Format> for ImageFormat {
    vk::Format::UNDEFINED => ImageFormat::Undefined,

    vk::Format::R4G4_UNORM_PACK8 => ImageFormat::R4G4UNormPack8,
    vk::Format::R4G4B4A4_UNORM_PACK16 => ImageFormat::R4G4B4A4UNormPack16,
    vk::Format::B4G4R4A4_UNORM_PACK16 => ImageFormat::B4G4R4A4UNormPack16,
    vk::Format::R5G6B5_UNORM_PACK16 => ImageFormat::R5G6B5UNormPack16,
    vk::Format::B5G6R5_UNORM_PACK16 => ImageFormat::B5G6R5UNormPack16,
    vk::Format::R5G5B5A1_UNORM_PACK16 => ImageFormat::R5G5B5A1UNormPack16,
    vk::Format::B5G5R5A1_UNORM_PACK16 => ImageFormat::B5G5R5A1UNormPack16,
    vk::Format::A1R5G5B5_UNORM_PACK16 => ImageFormat::A1R5G5B5UNormPack16,
    vk::Format::R8_UNORM => ImageFormat::R8UNorm,
    vk::Format::R8_SNORM => ImageFormat::R8SNorm,
    vk::Format::R8_USCALED => ImageFormat::R8UScaled,
    vk::Format::R8_SSCALED => ImageFormat::R8SScaled,
    vk::Format::R8_UINT => ImageFormat::R8UInt,
    vk::Format::R8_SINT => ImageFormat::R8SInt,
    vk::Format::R8_SRGB => ImageFormat::R8SRgb,
    vk::Format::R8G8_UNORM => ImageFormat::R8G8UNorm,
    vk::Format::R8G8_SNORM => ImageFormat::R8G8SNorm,
    vk::Format::R8G8_USCALED => ImageFormat::R8G8UScaled,
    vk::Format::R8G8_SSCALED => ImageFormat::R8G8SScaled,
    vk::Format::R8G8_UINT => ImageFormat::R8G8UInt,
    vk::Format::R8G8_SINT => ImageFormat::R8G8SInt,
    vk::Format::R8G8_SRGB => ImageFormat::R8G8SRgb,
    vk::Format::R8G8B8_UNORM => ImageFormat::R8G8B8UNorm,
    vk::Format::R8G8B8_SNORM => ImageFormat::R8G8B8SNorm,
    vk::Format::R8G8B8_USCALED => ImageFormat::R8G8B8UScaled,
    vk::Format::R8G8B8_SSCALED => ImageFormat::R8G8B8SScaled,
    vk::Format::R8G8B8_UINT => ImageFormat::R8G8B8UInt,
    vk::Format::R8G8B8_SINT => ImageFormat::R8G8B8SInt,
    vk::Format::R8G8B8_SRGB => ImageFormat::R8G8B8SRgb,
    vk::Format::B8G8R8_UNORM => ImageFormat::B8G8R8UNorm,
    vk::Format::B8G8R8_SNORM => ImageFormat::B8G8R8SNorm,
    vk::Format::B8G8R8_USCALED => ImageFormat::B8G8R8UScaled,
    vk::Format::B8G8R8_SSCALED => ImageFormat::B8G8R8SScaled,
    vk::Format::B8G8R8_UINT => ImageFormat::B8G8R8UInt,
    vk::Format::B8G8R8_SINT => ImageFormat::B8G8R8SInt,
    vk::Format::B8G8R8_SRGB => ImageFormat::B8G8R8SRgb,
    vk::Format::R8G8B8A8_UNORM => ImageFormat::R8G8B8A8UNorm,
    vk::Format::R8G8B8A8_SNORM => ImageFormat::R8G8B8A8SNorm,
    vk::Format::R8G8B8A8_USCALED => ImageFormat::R8G8B8A8UScaled,
    vk::Format::R8G8B8A8_SSCALED => ImageFormat::R8G8B8A8SScaled,
    vk::Format::R8G8B8A8_UINT => ImageFormat::R8G8B8A8UInt,
    vk::Format::R8G8B8A8_SINT => ImageFormat::R8G8B8A8SInt,
    vk::Format::R8G8B8A8_SRGB => ImageFormat::R8G8B8A8SRgb,
    vk::Format::B8G8R8A8_UNORM => ImageFormat::B8G8R8A8UNorm,
    vk::Format::B8G8R8A8_SNORM => ImageFormat::B8G8R8A8SNorm,
    vk::Format::B8G8R8A8_USCALED => ImageFormat::B8G8R8A8UScaled,
    vk::Format::B8G8R8A8_SSCALED => ImageFormat::B8G8R8A8SScaled,
    vk::Format::B8G8R8A8_UINT => ImageFormat::B8G8R8A8UInt,
    vk::Format::B8G8R8A8_SINT => ImageFormat::B8G8R8A8SInt,
    vk::Format::B8G8R8A8_SRGB => ImageFormat::B8G8R8A8SRgb,
    vk::Format::A8B8G8R8_UNORM_PACK32 => ImageFormat::A8B8G8R8UNormPack32,
    vk::Format::A8B8G8R8_SNORM_PACK32 => ImageFormat::A8B8G8R8SNormPack32,
    vk::Format::A8B8G8R8_USCALED_PACK32 => ImageFormat::A8B8G8R8UScaledPack32,
    vk::Format::A8B8G8R8_SSCALED_PACK32 => ImageFormat::A8B8G8R8SScaledPack32,
    vk::Format::A8B8G8R8_UINT_PACK32 => ImageFormat::A8B8G8R8UIntPack32,
    vk::Format::A8B8G8R8_SINT_PACK32 => ImageFormat::A8B8G8R8SIntPack32,
    vk::Format::A8B8G8R8_SRGB_PACK32 => ImageFormat::A8B8G8R8SRgbPack32,
    vk::Format::A2R10G10B10_UNORM_PACK32 => ImageFormat::A2R10G10B10UNormPack32,
    vk::Format::A2R10G10B10_SNORM_PACK32 => ImageFormat::A2R10G10B10SNormPack32,
    vk::Format::A2R10G10B10_USCALED_PACK32 => ImageFormat::A2R10G10B10UScaledPack32,
    vk::Format::A2R10G10B10_SSCALED_PACK32 => ImageFormat::A2R10G10B10SScaledPack32,
    vk::Format::A2R10G10B10_UINT_PACK32 => ImageFormat::A2R10G10B10UIntPack32,
    vk::Format::A2R10G10B10_SINT_PACK32 => ImageFormat::A2R10G10B10SIntPack32,
    vk::Format::A2B10G10R10_UNORM_PACK32 => ImageFormat::A2B10G10R10UNormPack32,
    vk::Format::A2B10G10R10_SNORM_PACK32 => ImageFormat::A2B10G10R10SNormPack32,
    vk::Format::A2B10G10R10_USCALED_PACK32 => ImageFormat::A2B10G10R10UScaledPack32,
    vk::Format::A2B10G10R10_SSCALED_PACK32 => ImageFormat::A2B10G10R10SScaledPack32,
    vk::Format::A2B10G10R10_UINT_PACK32 => ImageFormat::A2B10G10R10UIntPack32,
    vk::Format::A2B10G10R10_SINT_PACK32 => ImageFormat::A2B10G10R10SIntPack32,
    vk::Format::R16_UNORM => ImageFormat::R16UNorm,
    vk::Format::R16_SNORM => ImageFormat::R16SNorm,
    vk::Format::R16_USCALED => ImageFormat::R16UScaled,
    vk::Format::R16_SSCALED => ImageFormat::R16SScaled,
    vk::Format::R16_UINT => ImageFormat::R16UInt,
    vk::Format::R16_SINT => ImageFormat::R16SInt,
    vk::Format::R16_SFLOAT => ImageFormat::R16SFloat,
    vk::Format::R16G16_UNORM => ImageFormat::R16G16UNorm,
    vk::Format::R16G16_SNORM => ImageFormat::R16G16SNorm,
    vk::Format::R16G16_USCALED => ImageFormat::R16G16UScaled,
    vk::Format::R16G16_SSCALED => ImageFormat::R16G16SScaled,
    vk::Format::R16G16_UINT => ImageFormat::R16G16UInt,
    vk::Format::R16G16_SINT => ImageFormat::R16G16SInt,
    vk::Format::R16G16_SFLOAT => ImageFormat::R16G16SFloat,
    vk::Format::R16G16B16_UNORM => ImageFormat::R16G16B16UNorm,
    vk::Format::R16G16B16_SNORM => ImageFormat::R16G16B16SNorm,
    vk::Format::R16G16B16_USCALED => ImageFormat::R16G16B16UScaled,
    vk::Format::R16G16B16_SSCALED => ImageFormat::R16G16B16SScaled,
    vk::Format::R16G16B16_UINT => ImageFormat::R16G16B16UInt,
    vk::Format::R16G16B16_SINT => ImageFormat::R16G16B16SInt,
    vk::Format::R16G16B16_SFLOAT => ImageFormat::R16G16B16SFloat,
    vk::Format::R16G16B16A16_UNORM => ImageFormat::R16G16B16A16UNorm,
    vk::Format::R16G16B16A16_SNORM => ImageFormat::R16G16B16A16SNorm,
    vk::Format::R16G16B16A16_USCALED => ImageFormat::R16G16B16A16UScaled,
    vk::Format::R16G16B16A16_SSCALED => ImageFormat::R16G16B16A16SScaled,
    vk::Format::R16G16B16A16_UINT => ImageFormat::R16G16B16A16UInt,
    vk::Format::R16G16B16A16_SINT => ImageFormat::R16G16B16A16SInt,
    vk::Format::R16G16B16A16_SFLOAT => ImageFormat::R16G16B16A16SFloat,
    vk::Format::R32_UINT => ImageFormat::R32UInt,
    vk::Format::R32_SINT => ImageFormat::R32SInt,
    vk::Format::R32_SFLOAT => ImageFormat::R32SFloat,
    vk::Format::R32G32_UINT => ImageFormat::R32G32UInt,
    vk::Format::R32G32_SINT => ImageFormat::R32G32SInt,
    vk::Format::R32G32_SFLOAT => ImageFormat::R32G32SFloat,
    vk::Format::R32G32B32_UINT => ImageFormat::R32G32B32UInt,
    vk::Format::R32G32B32_SINT => ImageFormat::R32G32B32SInt,
    vk::Format::R32G32B32_SFLOAT => ImageFormat::R32G32B32SFloat,
    vk::Format::R32G32B32A32_UINT => ImageFormat::R32G32B32A32UInt,
    vk::Format::R32G32B32A32_SINT => ImageFormat::R32G32B32A32SInt,
    vk::Format::R32G32B32A32_SFLOAT => ImageFormat::R32G32B32A32SFloat,
    vk::Format::R64_UINT => ImageFormat::R64UInt,
    vk::Format::R64_SINT => ImageFormat::R64SInt,
    vk::Format::R64_SFLOAT => ImageFormat::R64SFloat,
    vk::Format::R64G64_UINT => ImageFormat::R64G64UInt,
    vk::Format::R64G64_SINT => ImageFormat::R64G64SInt,
    vk::Format::R64G64_SFLOAT => ImageFormat::R64G64SFloat,
    vk::Format::R64G64B64_UINT => ImageFormat::R64G64B64UInt,
    vk::Format::R64G64B64_SINT => ImageFormat::R64G64B64SInt,
    vk::Format::R64G64B64_SFLOAT => ImageFormat::R64G64B64SFloat,
    vk::Format::R64G64B64A64_UINT => ImageFormat::R64G64B64A64UInt,
    vk::Format::R64G64B64A64_SINT => ImageFormat::R64G64B64A64SInt,
    vk::Format::R64G64B64A64_SFLOAT => ImageFormat::R64G64B64A64SFloat,
    vk::Format::B10G11R11_UFLOAT_PACK32 => ImageFormat::B10G11R11UFloatPack32,
    vk::Format::E5B9G9R9_UFLOAT_PACK32 => ImageFormat::E5B9G9R9UFloatPack32,
    vk::Format::D16_UNORM => ImageFormat::D16UNorm,
    vk::Format::X8_D24_UNORM_PACK32 => ImageFormat::X8D24UNormPack32,
    vk::Format::D32_SFLOAT => ImageFormat::D32SFloat,
    vk::Format::S8_UINT => ImageFormat::S8UInt,
    vk::Format::D16_UNORM_S8_UINT => ImageFormat::D16UNormS8UInt,
    vk::Format::D24_UNORM_S8_UINT => ImageFormat::D24UNormS8UInt,
    vk::Format::D32_SFLOAT_S8_UINT => ImageFormat::D32SFloatS8UInt,
    vk::Format::BC1_RGB_UNORM_BLOCK => ImageFormat::BC1RGBUNormBlock,
    vk::Format::BC1_RGB_SRGB_BLOCK => ImageFormat::BC1RGBSRgbBlock,
    vk::Format::BC1_RGBA_UNORM_BLOCK => ImageFormat::BC1RGBAUNormBlock,
    vk::Format::BC1_RGBA_SRGB_BLOCK => ImageFormat::BC1RGBASRgbBlock,
    vk::Format::BC2_UNORM_BLOCK => ImageFormat::BC2UNormBlock,
    vk::Format::BC2_SRGB_BLOCK => ImageFormat::BC2SRgbBlock,
    vk::Format::BC3_UNORM_BLOCK => ImageFormat::BC3UNormBlock,
    vk::Format::BC3_SRGB_BLOCK => ImageFormat::BC3SRgbBlock,
    vk::Format::BC4_UNORM_BLOCK => ImageFormat::BC4UNormBlock,
    vk::Format::BC4_SNORM_BLOCK => ImageFormat::BC4SNormBlock,
    vk::Format::BC5_UNORM_BLOCK => ImageFormat::BC5UNormBlock,
    vk::Format::BC5_SNORM_BLOCK => ImageFormat::BC5SNormBlock,
    vk::Format::BC6H_UFLOAT_BLOCK => ImageFormat::BC6HUFloatBlock,
    vk::Format::BC6H_SFLOAT_BLOCK => ImageFormat::BC6HSFloatBlock,
    vk::Format::BC7_UNORM_BLOCK => ImageFormat::BC7UNormBlock,
    vk::Format::BC7_SRGB_BLOCK => ImageFormat::BC7SRgbBlock,
    vk::Format::ETC2_R8G8B8_UNORM_BLOCK => ImageFormat::ETC2R8G8B8UNormBlock,
    vk::Format::ETC2_R8G8B8_SRGB_BLOCK => ImageFormat::ETC2R8G8B8SRgbBlock,
    vk::Format::ETC2_R8G8B8A1_UNORM_BLOCK => ImageFormat::ETC2R8G8B8A1UNormBlock,
    vk::Format::ETC2_R8G8B8A1_SRGB_BLOCK => ImageFormat::ETC2R8G8B8A1SRgbBlock,
    vk::Format::ETC2_R8G8B8A8_UNORM_BLOCK => ImageFormat::ETC2R8G8B8A8UNormBlock,
    vk::Format::ETC2_R8G8B8A8_SRGB_BLOCK => ImageFormat::ETC2R8G8B8A8SRgbBlock,
    vk::Format::EAC_R11_UNORM_BLOCK => ImageFormat::EACR11UNormBlock,
    vk::Format::EAC_R11_SNORM_BLOCK => ImageFormat::EACR11SNormBlock,
    vk::Format::EAC_R11G11_UNORM_BLOCK => ImageFormat::EACR11G11UNormBlock,
    vk::Format::EAC_R11G11_SNORM_BLOCK => ImageFormat::EACR11G11SNormBlock,
    vk::Format::ASTC_4X4_UNORM_BLOCK => ImageFormat::ASTC4X4UNormBlock,
    vk::Format::ASTC_4X4_SRGB_BLOCK => ImageFormat::ASTC4X4SRgbBlock,
    vk::Format::ASTC_5X4_UNORM_BLOCK => ImageFormat::ASTC5X4UNormBlock,
    vk::Format::ASTC_5X4_SRGB_BLOCK => ImageFormat::ASTC5X4SRgbBlock,
    vk::Format::ASTC_5X5_UNORM_BLOCK => ImageFormat::ASTC5X5UNormBlock,
    vk::Format::ASTC_5X5_SRGB_BLOCK => ImageFormat::ASTC5X5SRgbBlock,
    vk::Format::ASTC_6X5_UNORM_BLOCK => ImageFormat::ASTC6X5UNormBlock,
    vk::Format::ASTC_6X5_SRGB_BLOCK => ImageFormat::ASTC6X5SRgbBlock,
    vk::Format::ASTC_6X6_UNORM_BLOCK => ImageFormat::ASTC6X6UNormBlock,
    vk::Format::ASTC_6X6_SRGB_BLOCK => ImageFormat::ASTC6X6SRgbBlock,
    vk::Format::ASTC_8X5_UNORM_BLOCK => ImageFormat::ASTC8X5UNormBlock,
    vk::Format::ASTC_8X5_SRGB_BLOCK => ImageFormat::ASTC8X5SRgbBlock,
    vk::Format::ASTC_8X6_UNORM_BLOCK => ImageFormat::ASTC8X6UNormBlock,
    vk::Format::ASTC_8X6_SRGB_BLOCK => ImageFormat::ASTC8X6SRgbBlock,
    vk::Format::ASTC_8X8_UNORM_BLOCK => ImageFormat::ASTC8X8UNormBlock,
    vk::Format::ASTC_8X8_SRGB_BLOCK => ImageFormat::ASTC8X8SRgbBlock,
    vk::Format::ASTC_10X5_UNORM_BLOCK => ImageFormat::ASTC10X5UNormBlock,
    vk::Format::ASTC_10X5_SRGB_BLOCK => ImageFormat::ASTC10X5SRgbBlock,
    vk::Format::ASTC_10X6_UNORM_BLOCK => ImageFormat::ASTC10X6UNormBlock,
    vk::Format::ASTC_10X6_SRGB_BLOCK => ImageFormat::ASTC10X6SRgbBlock,
    vk::Format::ASTC_10X8_UNORM_BLOCK => ImageFormat::ASTC10X8UNormBlock,
    vk::Format::ASTC_10X8_SRGB_BLOCK => ImageFormat::ASTC10X8SRgbBlock,
    vk::Format::ASTC_10X10_UNORM_BLOCK => ImageFormat::ASTC10X10UNormBlock,
    vk::Format::ASTC_10X10_SRGB_BLOCK => ImageFormat::ASTC10X10SRgbBlock,
    vk::Format::ASTC_12X10_UNORM_BLOCK => ImageFormat::ASTC12X10UNormBlock,
    vk::Format::ASTC_12X10_SRGB_BLOCK => ImageFormat::ASTC12X10SRgbBlock,
    vk::Format::ASTC_12X12_UNORM_BLOCK => ImageFormat::ASTC12X12UNormBlock,
    vk::Format::ASTC_12X12_SRGB_BLOCK => ImageFormat::ASTC12X12SRgbBlock,
});



/// The layout of an Image.
#[derive(Clone, Copy, Debug)]
pub enum ImageLayout {
    /// We don't care about the layout / it's not yet defined.
    Undefined,
    /// The image has a default layout and _may_ contain data, but its layout is not yet initialized.
    /// 
    /// This can only be used for the initialLayout in the VkImageCreateInfo struct.
    Preinitialized,
    /// A general layout that is applicable to many things (i.e., all types of device access, though probably not optimized).
    General,

    /// Optimal layout for colour attachments.
    ColourAttachment,
    /// Optimal layout for a depth stencil.
    DepthStencil,
    /// Optimal layout for a read-only depth stencil.
    DepthStencilReadOnly,
    /// Optimal layout for an image that is read during a shader stage.
    ShaderReadOnly,
    /// Optimal layout for presenting to a swapchain.
    Present,

    /// Optimal layout for the image data being transferred to another image.
    TransferSrc,
    /// Optimal layout for the image's data being overwritten with transferred data from another image.
    TransferDst,
}

enum_from!(impl From<vk::ImageLayout> for ImageLayout {
    vk::ImageLayout::UNDEFINED      => ImageLayout::Undefined,
    vk::ImageLayout::PREINITIALIZED => ImageLayout::Preinitialized,
    vk::ImageLayout::GENERAL        => ImageLayout::General,

    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL         => ImageLayout::ColourAttachment,
    vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => ImageLayout::DepthStencil,
    vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL  => ImageLayout::DepthStencilReadOnly,
    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL         => ImageLayout::ShaderReadOnly,
    vk::ImageLayout::PRESENT_SRC_KHR                  => ImageLayout::Present,

    vk::ImageLayout::TRANSFER_SRC_OPTIMAL => ImageLayout::TransferSrc,
    vk::ImageLayout::TRANSFER_DST_OPTIMAL => ImageLayout::TransferDst,
});



/// Defines how we might use an Image.
#[derive(Clone, Copy, Debug)]
pub enum ImageAspect {
    /// The image will be used as a colour attachment.
    Colour,
    /// The image will be used as a Depth stencil.
    Depth,
    /// The image will be used as a gemeral stencil.
    Stencil,
    /// The image will be used to carry metadata.
    Metadata,
}

impl Display for ImageAspect {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ImageAspect::*;
        match self {
            Colour   => write!(f, "Colour"),
            Depth    => write!(f, "Depth"),
            Stencil  => write!(f, "Stencil"),
            Metadata => write!(f, "Metadata"),
        }
    }
}

enum_from!(impl From<vk::ImageAspectFlags> for ImageAspect {
    vk::ImageAspectFlags::COLOR    => ImageAspect::Colour,
    vk::ImageAspectFlags::DEPTH    => ImageAspect::Depth,
    vk::ImageAspectFlags::STENCIL  => ImageAspect::Stencil,
    vk::ImageAspectFlags::METADATA => ImageAspect::Metadata,
});
