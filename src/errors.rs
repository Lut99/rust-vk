//  ERRORS.rs
//    by Lut99
// 
//  Created:
//    26 Mar 2022, 14:09:56
//  Last edited:
//    06 Aug 2022, 10:55:21
//  Auto updated?
//    Yes
// 
//  Description:
//!   Collects all errors for the crate.
// 

use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;

use ash::vk;


/***** ERRORS *****/
/// Defines error(s) relating to the extension & layer enums.
#[derive(Debug)]
pub enum ExtensionError {
    /// The given string value was not a valid one for the InstanceExtension.
    UnknownInstanceExtension{ got: String },
    /// The given string value was not a valid one for the InstanceLayer.
    UnknownInstanceLayer{ got: String },
    /// The given string value was not a valid one for the DeviceExtension.
    UnknownDeviceExtension{ got: String },
    /// The given string value was not a valid one for the DeviceLayer.
    UnknownDeviceLayer{ got: String },
}

impl Display for ExtensionError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ExtensionError::*;
        match self {
            UnknownInstanceExtension{ got } => write!(f, "Unknown instance extension '{}'", got),
            UnknownInstanceLayer{ got }     => write!(f, "Unknown instance layer '{}'", got),
            UnknownDeviceExtension{ got }   => write!(f, "Unknown device extension '{}'", got),
            UnknownDeviceLayer{ got }       => write!(f, "Unknown device layer '{}'", got),
        }
    }
}

impl Error for ExtensionError {}



/// Defines errors relating to going back and forth between AttributeLayouts and vk::Formats.
#[derive(Clone, Debug)]
pub enum AttributeLayoutError {
    /// Given vk::Format value was a valid vk::Format, but not a valid AttributeLayout
    IllegalFormatValue{ value: vk::Format },
}

impl Display for AttributeLayoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use AttributeLayoutError::*;
        match self {
            IllegalFormatValue{ value } => write!(f, "Encountered valid vk::Format value '{}' ({:?}), but that value is illegal for an AttributeLayout", value.as_raw(), value),
        }
    }
}

impl Error for AttributeLayoutError {}



/// Defines errors that occur when setting up an Instance.
#[derive(Debug)]
pub enum InstanceError {
    /// Could not load the Vulkan library at runtime
    LoadError{ err: ash::LoadingError },
    /// Could not enumerate the extension properties (possible the extensions from a certain layer)
    ExtensionEnumerateError{ layer: Option<CString>, err: ash::vk::Result },
    /// Could not enumerate the layer properties
    LayerEnumerateError{ err: ash::vk::Result },
    /// Unknown extension encountered
    UnknownExtension{ extension: CString },
    /// Unknown layer encountered
    UnknownLayer{ layer: CString },

    /// Could not create the Instance
    CreateError{ err: ash::vk::Result },
    /// Could not create the debug messenger
    DebugCreateError{ err: ash::vk::Result },
}

impl Display for InstanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use InstanceError::*;
        match self {
            LoadError{ err }                      => write!(f, "Could not load the Vulkan library: {}", err),
            ExtensionEnumerateError{ layer, err } => write!(f, "Could not enumerate extensions properties{}: {}", if let Some(layer) = layer { format!(" for layer '{:?}'", layer) } else { String::new() }, err),
            LayerEnumerateError{ err }            => write!(f, "Could not enumerate layer properties: {}", err),
            UnknownExtension{ extension }         => write!(f, "Extension '{:?}' is not found in local Vulkan installation", extension),
            UnknownLayer{ layer }                 => write!(f, "Layer '{:?}' is not found in local Vulkan installation", layer),

            CreateError{ err }      => write!(f, "Could not create Vulkan instance: {}", err),
            DebugCreateError{ err } => write!(f, "Could not create Vulkan debug messenger: {}", err),
        }
    }
}

impl Error for InstanceError {}



/// Defines errors that occur when setting up an Instance.
#[derive(Clone, Debug)]
pub enum DeviceError {
    /// Could not enumerate over the available device extensions
    DeviceExtensionEnumerateError{ err: ash::vk::Result },
    /// The given device extension was not supported by the given device
    UnsupportedDeviceExtension{ index: usize, name: String, extension: CString },
    /// Could not enumerate over the available device layers
    DeviceLayerEnumerateError{ err: ash::vk::Result },
    /// The given device layer was not supported by the given device
    UnsupportedDeviceLayer{ index: usize, name: String, layer: CString },
    /// The given device feature was not supported by the given device
    UnsupportedFeature{ index: usize, name: String, feature: &'static str },

    /// Could not get the iterator over the physical devices
    PhysicalDeviceEnumerateError{ err: ash::vk::Result },
    /// Did not find the given physical device
    PhysicalDeviceNotFound{ index: usize },
    /// Could not convert the raw name of the device to a String
    PhysicalDeviceNameError{ index: usize, err: std::str::Utf8Error },
    /// Could not get the family info of the device.
    QueueFamilyError{ index: usize, err: QueueError },
    /// Could not create the new logical device
    DeviceCreateError{ err: ash::vk::Result },

    /// Could not wait for a Queue to be idle
    QueueIdleError{ err: QueueError },
    /// Could not wait for the Device to be idle
    DeviceIdleError{ err: ash::vk::Result },

    /// None of the found devices support this application
    NoSupportedPhysicalDevices,

    /// Could not get whether or not the given surface is supported
    SurfaceSupportError{ err: ash::vk::Result },
    /// Could not get the capabilities of the given surface
    SurfaceCapabilitiesError{ err: ash::vk::Result },
    /// Could not get the formats of the given surface
    SurfaceFormatsError{ err: ash::vk::Result },
    /// Could not get the present modes of the given surface
    SurfacePresentModesError{ err: ash::vk::Result },
    /// The given surface is not supported at all
    UnsupportedSurface,
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DeviceError::*;
        match self {
            DeviceExtensionEnumerateError{ err }                 => write!(f, "Could not enumerate device extension properties: {}", err),
            UnsupportedDeviceExtension{ index, name, extension } => write!(f, "Physical device {} ({}) does not support extension '{:?}'; choose another device", index, name, extension),
            DeviceLayerEnumerateError{ err }                     => write!(f, "Could not enumerate device layer properties: {}", err),
            UnsupportedDeviceLayer{ index, name, layer }         => write!(f, "Physical device {} ({}) does not support layer '{:?}'; choose another device", index, name, layer),
            UnsupportedFeature{ index, name, feature }           => write!(f, "Physical device {} ({}) does not support feature '{}'; choose another device", index, name, feature),

            PhysicalDeviceEnumerateError{ err }   => write!(f, "Could not enumerate physical devices: {}", err),
            PhysicalDeviceNotFound{ index }       => write!(f, "Could not find physical device '{}'; see the list of available devices by running 'list'", index),
            PhysicalDeviceNameError{ index, err } => write!(f, "Could not parse name of device {} as UTF-8: {}", index, err),
            QueueFamilyError{ index, err }        => write!(f, "Could not get the queue family info of device {}: {}", index, err),
            DeviceCreateError{ err }              => write!(f, "Could not create logical device: {}", err),

            QueueIdleError{ err }  => write!(f, "Could not wait for queue to be idle: {}", err),
            DeviceIdleError{ err } => write!(f, "Could not wait for device to be idle: {}", err),

            NoSupportedPhysicalDevices => write!(f, "No device found that supports this application"),

            SurfaceSupportError{ err }      => write!(f, "Could not query swapchain support for surface: {}", err),
            SurfaceCapabilitiesError{ err } => write!(f, "Could not query supported swapchain capabilities for surface: {}", err),
            SurfaceFormatsError{ err }      => write!(f, "Could not query supported swapchain formats for surface: {}", err),
            SurfacePresentModesError{ err } => write!(f, "Could not query supported swapchain present modes for surface: {}", err),
            UnsupportedSurface              => write!(f, "The given surface is not supported by the chosen device"),
        }
    }
}

impl Error for DeviceError {}



/// Defines errors relating to Queue properties and management.
#[derive(Clone, Debug)]
pub enum QueueError {
    /// One of the operations we want for the queue families is unsupported
    OperationUnsupported{ index: usize, name: String, operation: ash::vk::QueueFlags },

    /// Could not reset a fence
    FenceResetError{ err: SyncError },
    /// Could not submit the command buffer for rendering
    SubmitError{ err: ash::vk::Result },

    /// Could not wait for the queue to be idle
    IdleError{ err: ash::vk::Result },
}

impl Display for QueueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use QueueError::*;
        match self {
            OperationUnsupported{ index, name, operation } => write!(f, "Physical device {} ({}) does not have queues that support '{:?}'; choose another device", index, name, operation),

            FenceResetError{ err } => write!(f, "Could not reset Fence: {}", err),
            SubmitError{ err }     => write!(f, "Could not submit command buffer: {}", err),

            IdleError{ err } => write!(f, "Could not wait for queue to become idle: {}", err),
        }
    }
}

impl Error for QueueError {}



/// Defines errors that occur when setting up a Surface.
#[derive(Clone, Debug)]
pub enum SurfaceError {
    /// Could not create a new Windows surface
    WindowsSurfaceKHRCreateError{ err: ash::vk::Result },
    /// Could not create a new macOS surface
    MacOSSurfaceKHRCreateError{ err: ash::vk::Result },
    /// This linux installation does not use X11 or Wayland
    UnsupportedWindowSystem,
    /// Could not create a new X11 surface
    X11SurfaceKHRCreateError{ err: ash::vk::Result },
    /// Could not create a new Wayland surface
    WaylandSurfaceCreateError{ err: ash::vk::Result },
}

impl Display for SurfaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use SurfaceError::*;
        match self {
            WindowsSurfaceKHRCreateError{ err } => write!(f, "Could not create new Windows SurfaceKHR: {}", err),
            MacOSSurfaceKHRCreateError{ err }   => write!(f, "Could not create new macOS SurfaceKHR: {}", err),
            UnsupportedWindowSystem             => write!(f, "Target window is not an X11 or Wayland window; other window systems are not supported"),
            X11SurfaceKHRCreateError{ err }     => write!(f, "Could not create new X11 SurfaceKHR: {}", err),
            WaylandSurfaceCreateError{ err }    => write!(f, "Could not create new Wayland SurfaceKHR: {}", err),
        }
    }
}

impl Error for SurfaceError {}



/// Defines errors that occur when setting up a Surface.
#[derive(Clone, Debug)]
pub enum SwapchainError {
    /// The given surface was not supported at all by the given GPU.
    DeviceSurfaceSupportError{ index: usize, name: String, err: DeviceError },
    /// Could not find an appropriate format for this GPU / surface combo.
    NoFormatFound,
    /// Could not deduce any of the Swapchain properties.
    SwapchainDeduceError{ err: Box<Self> },
    /// Could not create a new swapchain
    SwapchainCreateError{ err: ash::vk::Result },
    /// Could not get the images from the swapchain
    SwapchainImagesError{ err: ash::vk::Result },
    /// Could not create an Image around one of the swapchain's images.
    ImageError{ err: ImageError },

    /// Could not get the next available image in the swapchain
    SwapchainNextImageError{ err: ash::vk::Result },

    /// Could not present a given image in the swapchain.
    SwapchainPresentError{ index: u32, err: ash::vk::Result },

    /// Could not wait for the device to become idle.
    DeviceIdleError{ err: DeviceError },
}

impl Display for SwapchainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use SwapchainError::*;
        match self {
            DeviceSurfaceSupportError{ index, name, err } => write!(f, "Device {} ('{}') does not support given Surface: {}", index, name, err),
            NoFormatFound                                 => write!(f, "No suitable formats found for swapchain; try choosing another device."),
            SwapchainDeduceError{ err }                   => write!(f, "Could not deduce Swapchain properties: {}", err),
            SwapchainCreateError{ err }                   => write!(f, "Could not create Swapchain: {}", err),
            SwapchainImagesError{ err }                   => write!(f, "Could not get Swapchain images: {}", err),
            ImageError{ err }                             => write!(f, "Could not create Image from swapchain image: {}", err),

            SwapchainNextImageError{ err } => write!(f, "Could not get next swapchain image: {}", err),

            SwapchainPresentError{ index, err } => write!(f, "Could not present swapchain image {}: {}", index, err),

            DeviceIdleError{ err } => write!(f, "{}", err),
        }
    }
}

impl Error for SwapchainError {}



/// Defines errors that relate to the Shader loading/compiling.
#[derive(Debug)]
pub enum ShaderError {
    /// Could not create a new module
    ShaderCreateError{ err: ash::vk::Result },

    /// Could not open the given shader file
    FileOpenError{ path: PathBuf, err: std::io::Error },
    /// Could not read from the given shader file
    FileReadError{ path: PathBuf, err: std::io::Error },

    /// Could not unpack an embedded file
    EmbeddedError,
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ShaderError::*;
        match self {
            ShaderCreateError{ err } => write!(f, "Could not create the ShaderModule: {}", err),

            FileOpenError{ path, err } => write!(f, "Could not open given SPIR-V shader file '{}': {}", path.display(), err),
            FileReadError{ path, err } => write!(f, "Could not read given SPIR-V shader file '{}': {}", path.display(), err),

            EmbeddedError => write!(f, "Could not load embedded shader code"),
        }
    }
}

impl Error for ShaderError {}



/// Defines errors that relate to DescriptorSets and DescriptorSetLayouts.
#[derive(Clone, Debug)]
pub enum DescriptorError {
    /// Could not create a new layout
    DescriptorSetLayoutCreateError{ err: ash::vk::Result },
}

impl Display for DescriptorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DescriptorError::*;
        match self {
            DescriptorSetLayoutCreateError{ err } => write!(f, "Could not create new DescriptorSetLayout: {}", err),
        }
    }
}

impl Error for DescriptorError {}



/// Defines errors that relate to a PipelineLayout.
#[derive(Clone, Debug)]
pub enum PipelineLayoutError {
    /// Could not create the PipelineLayout struct
    PipelineLayoutCreateError{ err: ash::vk::Result },
}

impl Display for PipelineLayoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use PipelineLayoutError::*;
        match self {
            PipelineLayoutCreateError{ err }      => write!(f, "Could not create new PipelineLayout: {}", err),
        }
    }
}

impl Error for PipelineLayoutError {}



/// Defines errors that relate to a RenderPass.
#[derive(Clone, Debug)]
pub enum RenderPassError {
    /// Could not create a RenderPass.
    RenderPassCreateError{ err: ash::vk::Result },
}

impl Display for RenderPassError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use RenderPassError::*;
        match self {
            RenderPassCreateError{ err } => write!(f, "Could not create new RenderPass: {}", err),
        }
    }
}

impl Error for RenderPassError {}



/// Defines errors that relate to a Pipeline.
#[derive(Debug)]
pub enum PipelineError {
    /// Could not open the PipelineCache file
    PipelineCacheOpenError{ path: PathBuf, err: std::io::Error },
    /// Could not read the PipelineCache file
    PipelineCacheReadError{ path: PathBuf, err: std::io::Error },
    /// Could not create a new PipelineCache
    PipelineCacheCreateError{ err: ash::vk::Result },

    /// The given PipelineCache result was not a success
    PipelineCacheError{ err: Box<Self> },
    /// The given Shader result was not a success
    ShaderError{ err: ShaderError },
    /// Could not create the final Pipeline struct
    PipelineCreateError{ err: ash::vk::Result },
}

impl Display for PipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use PipelineError::*;
        match self {
            PipelineCacheOpenError{ path, err } => write!(f, "Could not open pipeline cache file '{}': {}", path.display(), err),
            PipelineCacheReadError{ path, err } => write!(f, "Could not read pipeline cache file '{}': {}", path.display(), err),
            PipelineCacheCreateError{ err }     => write!(f, "Could not create new PipelineCache: {}", err),

            PipelineCacheError{ err }  => write!(f, "Given PipelineCache constructor call was a fail: {}", err),
            ShaderError{ err }         => write!(f, "Given Shader constructor call was a fail: {}", err),
            PipelineCreateError{ err } => write!(f, "Could not create new Pipeline: {}", err),
        }
    }
}

impl Error for PipelineError {}



/// Defines errors that relate to an Image.
#[derive(Clone, Debug)]
pub enum ImageError {
    /// Temporary placeholder error
    Temp,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ImageError::*;
        match self {
            Temp => write!(f, "<TEMP>"),
        }
    }
}

impl Error for ImageError {}



/// Defines errors that relate to an ImageView.
#[derive(Clone, Debug)]
pub enum ImageViewError {
    /// Could not construct the image view
    ViewCreateError{ err: ash::vk::Result },
}

impl Display for ImageViewError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ImageViewError::*;
        match self {
            ViewCreateError{ err } => write!(f, "Could not create ImageView: {}", err),
        }
    }
}

impl Error for ImageViewError {}



/// Defines errors that relate to framebuffers
#[derive(Clone, Debug)]
pub enum FramebufferError {
    /// Could not create a new Framebuffer
    FramebufferCreateError{ err: ash::vk::Result },
}

impl Display for FramebufferError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use FramebufferError::*;
        match self {
            FramebufferCreateError{ err } => write!(f, "Could not create Framebuffer: {}", err),
        }
    }
}

impl Error for FramebufferError {}



/// Defines errors for synchronization primitives
#[derive(Clone, Debug)]
pub enum SyncError {
    /// Could not create a new Semaphore
    SemaphoreCreateError{ err: ash::vk::Result },
    /// Could not create a new Fence
    FenceCreateError{ err: ash::vk::Result },

    /// The given Fence has timed-out.
    FenceTimeout{ timeout: u64 },
    /// Could not wait for a Fence.
    FenceWaitError{ err: ash::vk::Result },

    /// Could not reset a Fence.
    FenceResetError{ err: ash::vk::Result },
}

impl Display for SyncError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use SyncError::*;
        match self {
            SemaphoreCreateError{ err } => write!(f, "Could not create Sempahore: {}", err),
            FenceCreateError{ err }     => write!(f, "Could not create Fence: {}", err),
            
            FenceTimeout{ timeout } => write!(f, "Fence timed-out after {} milliseconds", timeout),
            FenceWaitError{ err }   => write!(f, "Could not wait for Fence: {}", err),
            
            FenceResetError{ err } => write!(f, "Could not reset Fence: {}", err),
        }
    }
}

impl Error for SyncError {}
