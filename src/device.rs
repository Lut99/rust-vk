//  DEVICE.rs
//    by Lut99
// 
//  Created:
//    27 Mar 2022, 13:19:36
//  Last edited:
//    13 Aug 2022, 17:21:47
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the Device struct, which handles both physical and
//!   logical
// 

use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

use ash::vk;

use crate::{debug, to_cstring};
pub use crate::errors::DeviceError as Error;
use crate::log_destroy;
use crate::auxillary::enums::{DeviceKind, QueueKind};
use crate::auxillary::structs::{DeviceFeatures, DeviceInfo, PhysicalDeviceProperties, QueueFamilyInfo, SwapchainSupport};
use crate::instance::Instance;
use crate::surface::Surface;
use crate::queue::Queues;


/***** HELPER FUNCTIONS *****/
/// Checks if the given physical device supports the given lists of device extensions, device layers and device features.
/// 
/// # Errors
/// 
/// This function returns errors if the given device does not support all of the required extensions, layers and features.
fn supports(
    instance: &Rc<Instance>,
    physical_device: vk::PhysicalDevice,
    physical_device_index: usize,
    physical_device_name: &str,
    p_device_extensions: &[*const i8],
    p_device_layers: &[*const i8],
    _features: &vk::PhysicalDeviceFeatures,
) -> Result<(), Error> {
    // Test if all of the given extensions are supported on this device
    let avail_extensions = match unsafe { instance.enumerate_device_extension_properties(physical_device) } {
        Ok(extensions) => extensions,
        Err(err)       => { return Err(Error::DeviceExtensionEnumerateError{ err }); }
    };
    for req_ext in p_device_extensions {
        // Cast it to a CStr
        let req_ext: &CStr = unsafe { &CStr::from_ptr(*req_ext) };

        // Iterate through the available extensions
        let mut found = false;
        for avail_ext in &avail_extensions {
            // Make sure it's a CStr
            let avail_ext: &CStr = unsafe { &CStr::from_ptr(avail_ext.extension_name.as_ptr()) };

            // Compare them
            if req_ext == avail_ext { found = true; break; }
        }

        // If still not found, error
        if !found { return Err(Error::UnsupportedDeviceExtension{ index: physical_device_index, name: physical_device_name.to_string(), extension: req_ext.to_owned() }); }
    }

    // Next, test if all layers are supported
    let avail_layers = match unsafe { instance.enumerate_device_layer_properties(physical_device) } {
        Ok(layers) => layers,
        Err(err)   => { return Err(Error::DeviceLayerEnumerateError{ err }); }
    };
    for req_lay in p_device_layers {
        // Cast it to a CStr
        let req_lay: &CStr = unsafe { &CStr::from_ptr(*req_lay) };

        // Iterate through the available extensions
        let mut found = false;
        for avail_lay in &avail_layers {
            // Make sure it's a CStr
            let avail_lay: &CStr = unsafe { &CStr::from_ptr(avail_lay.layer_name.as_ptr()) };

            // Compare them
            if req_lay == avail_lay { found = true; break; }
        }

        // If still not found, error
        if !found { return Err(Error::UnsupportedDeviceLayer{ index: physical_device_index, name: physical_device_name.to_string(), layer: req_lay.to_owned() }); }
    }

    // Finally, test if features are supported
    let _avail_features: vk::PhysicalDeviceFeatures = unsafe { instance.get_physical_device_features(physical_device) };
    /* TODO */

    // We support it
    Ok(())
}





/***** POPULATE FUNCTIONS *****/
/// Populates a DeviceQueueCreateInfo struct.
/// 
/// Uses the given parameters to describe a new set of queues from a single queue family.
/// 
/// The number of queues we will construct for this family depends on the length of the given queue_priorities list.
#[inline]
fn populate_queue_info(family_index: u32, queue_priorities: &[f32]) -> vk::DeviceQueueCreateInfo {
    vk::DeviceQueueCreateInfo {
        // Define the often-used fields on these structs
        s_type : vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::DeviceQueueCreateFlags::empty(),

        // Define to which queue family the new queues belong
        queue_family_index : family_index,
        // Define the queue priorities. The length of this list determines how many queues.
        p_queue_priorities : queue_priorities.as_ptr(),
        queue_count        : queue_priorities.len() as u32,
    }
}

/// Populates a DeviceCreateInfo struct.
/// 
/// Uses the given properties to initialize a DeviceCreateInfo struct. Some checks are done beforehand, like if all extensions / layers / features are supported on this device.
/// 
/// # Errors
/// 
/// Error only occur when the given device does not support all of the given extensions / layers / features.
fn populate_device_info(
    instance: &Rc<Instance>,
    physical_device: vk::PhysicalDevice,
    physical_device_index: usize,
    physical_device_name: &str,
    queue_infos: &[vk::DeviceQueueCreateInfo],
    p_device_extensions: &[*const i8],
    p_device_layers: &[*const i8],
    features: &vk::PhysicalDeviceFeatures,
) -> Result<vk::DeviceCreateInfo, Error> {
    // Make sure that the physical device supports everything
    supports(instance, physical_device, physical_device_index, physical_device_name, p_device_extensions, p_device_layers, features)?;

    // With the checks complete, throw everything in the resulting struct
    Ok(vk::DeviceCreateInfo {
        // Do the standard stuff
        s_type : vk::StructureType::DEVICE_CREATE_INFO,
        p_next : ptr::null(),
        flags  : vk::DeviceCreateFlags::empty(),

        // Define the queue create infos
        p_queue_create_infos    : queue_infos.as_ptr(),
        queue_create_info_count : queue_infos.len() as u32,

        // Define the extensions
        pp_enabled_extension_names : p_device_extensions.as_ptr(),
        enabled_extension_count    : p_device_extensions.len() as u32,

        // Define the layer
        pp_enabled_layer_names : p_device_layers.as_ptr(),
        enabled_layer_count    : p_device_layers.len() as u32,

        // Finally, define the features
        p_enabled_features : features,
    })
}





/***** LIBRARY *****/
/// The Device struct provides logic to work with both Vulkan's PhysicalDevices and Devices.
pub struct Device {
    /// The instance used to initialize the device
    instance        : Rc<Instance>,
    /// The PhysicalDevice around which we wrap.
    physical_device : vk::PhysicalDevice,
    /// The logical Device around which we wrap.
    device          : Rc<ash::Device>,
    /// The queues for the internal device.
    queues          : Queues,

    /// The index of the device
    index    : usize,
    /// The PhysicalDevice properties that stores information about the device.
    props    : PhysicalDeviceProperties,
    // /// The name of the device
    // name     : String,
    // /// The type of the device (as a String as well)
    // kind     : DeviceKind,
    /// The QueueFamilyInfo that describes the queue families for this device.
    families : QueueFamilyInfo,
}

impl Device {
    /// Constructor for the Device.
    /// 
    /// The Device class is meant to provide access to both a PhysicalDevice and Vulkan's abstraction over it.
    /// 
    /// # Arguments
    /// - `instance`: An Rc of the global instance that we may use to initialize the device.
    /// - `physical_device_index`: The index of the physical device we want to wrap around. Can be obtained by using Device::auto_select().
    /// - `device_extensions`: A slice of Device extensions to enable on the Device.
    /// - `device_layers`: A slice of Device layers to enable on the Device.
    /// - `device_features`: A DeviceFeatures struct that describes the features to enable on the Device.
    /// 
    /// # Returns
    /// Returns a new Device instance on success, or else an Error describing what went wrong if the Device creation failed.
    pub fn new(instance: Rc<Instance>, physical_device_index: usize, device_extensions: &[&str], device_layers: &[&str], device_features: &DeviceFeatures) -> Result<Rc<Self>, Error> {
        // We enumerate through all the physical devices to find the appropriate one
        let physical_devices = match unsafe { instance.enumerate_physical_devices() } {
            Ok(devices) => devices,
            Err(err)    => { return Err(Error::PhysicalDeviceEnumerateError{ err }); }  
        };
        let mut target_physical_device: Option<vk::PhysicalDevice> = None;
        for (i, physical_device) in physical_devices.iter().enumerate() {
            // Check if this has the index we want
            if i == physical_device_index {
                // It is; we'll take it
                target_physical_device = Some(*physical_device);
            }
        }
        let physical_device = match target_physical_device {
            Some(device) => device,
            None         => { return Err(Error::PhysicalDeviceNotFound{ index: physical_device_index }); }
        };
    


        // Get the properties of this device
        let device_properties: PhysicalDeviceProperties = unsafe { instance.get_physical_device_properties(physical_device) }.into();

        // // Get a readable name and type
        // let device_name: String = match unsafe { CStr::from_ptr(device_properties.device_name.as_ptr()) }.to_str() {
        //     Ok(name) => name.to_string(),
        //     Err(err) => { return Err(Error::PhysicalDeviceNameError{ index: physical_device_index, err }); }
        // };
        // let device_type: DeviceKind = device_properties.device_type.into();



        // Collect the queue families for this device
        let family_info = match QueueFamilyInfo::new(&instance, physical_device, physical_device_index, &device_properties.name) {
            Ok(info) => info,
            Err(err) => { return Err(Error::QueueFamilyError{ index: physical_device_index, err }); }
        };



        // Do some debug prints about the selected device
        debug!("Using physical device {} '{}' ({})", physical_device_index, &device_properties.name, &device_properties.kind);
        debug!("Selected queue families:");
        debug!(" - Graphics : {}", family_info.graphics);
        debug!(" - Memory   : {}", family_info.memory);
        debug!(" - Compute  : {}", family_info.compute);



        // Prepare getting the queues from the device
        let queue_priorities = vec![ 1.0 ];
        let queue_infos: Vec<vk::DeviceQueueCreateInfo> = family_info.unique().map(|family| populate_queue_info(family, &queue_priorities)).collect();



        // Map the given device extensions and layers to pointers
        let device_extensions: Vec<CString> = device_extensions.iter().map(|extension| to_cstring!(extension)).collect();
        #[cfg(target_os = "macos")]
        let device_extensions = {
            let mut device_extensions = device_extensions;
            device_extensions.push(DeviceExtension::PortabilitySubset.into());
            device_extensions
        };
        let device_layers: Vec<CString>     = device_layers.iter().map(|layer| to_cstring!(layer)).collect();
        let p_device_extensions: Vec<*const i8> = (0..device_extensions.len()).map(|i| device_extensions[i].as_ptr()).collect();
        let p_device_layers: Vec<*const i8>     = (0..device_layers.len()).map(|i| device_layers[i].as_ptr()).collect();



        // Create the DeviceCreateInfo with all this
        let vk_device_features: vk::PhysicalDeviceFeatures = device_features.into();
        let device_info = populate_device_info(&instance, physical_device, physical_device_index, &device_properties.name, &queue_infos, &p_device_extensions, &p_device_layers, &vk_device_features)?;

        // Use that to create the device
        debug!("Initializing device...");
        let device: ash::Device = unsafe {
            match instance.create_device(physical_device, &device_info, None) {
                Ok(device) => device,
                Err(err)   => { return Err(Error::DeviceCreateError{ err }); }
            }
        };

        // Get the queues
        let device = Rc::new(device);
        let queues = Queues::new(&device, &family_info);



        // Done! Return the new GPU
        Ok(Rc::new(Self {
            instance,
            physical_device,
            device,
            queues,

            index    : physical_device_index,
            props    : device_properties,
            families : family_info,
        }))
    }



    /// Wait until the device is idle.
    /// 
    /// # Arguments
    /// - `queue`: If given, waits until the given queue is idle in the Device instead of all queues.
    #[inline]
    pub fn drain(&self, queue: Option<QueueKind>) -> Result<(), Error> {
        match queue {
            // In all Some-cases, just wait for that queue
            Some(QueueKind::Graphics) => self.queues.graphics.drain().map_err(|err| Error::QueueIdleError{ err }),
            Some(QueueKind::Memory)   => self.queues.memory.drain().map_err(|err| Error::QueueIdleError{ err }),
            Some(QueueKind::Present)  => self.queues.present.drain().map_err(|err| Error::QueueIdleError{ err }),
            Some(QueueKind::Compute)  => self.queues.compute.drain().map_err(|err| Error::QueueIdleError{ err }),

            // Otherwise, wait for the device
            None => match unsafe { self.device.device_wait_idle() } {
                Ok(_)    => Ok(()),
                Err(err) => Err(Error::DeviceIdleError{ err }),
            }
        }
    }



    /// Tries to automatically select the best GPU.
    /// 
    /// Iterates through all the GPUs that can be found in the given instance, and then tries to select the most appropriate one for the Game.
    /// 
    /// # Arguments
    /// - `instance`: The Instance object to seRch for GPUs in.
    /// - `device_extensions`: A slice of extensions that the GPU should support.
    /// - `device_layers`: A slice of layers that the GPU should support.
    /// - `device_features`: A struct of features that the GPU should support.
    /// 
    /// # Returns
    /// The index of the chosen GPU if we could find one, or, either if we did not find one or we failed otherwise, an Error detailing what went wrong.
    pub fn auto_select(instance: Rc<Instance>, device_extensions: &[&str], device_layers: &[&str], device_features: &DeviceFeatures) -> Result<usize, Error> {
        // Map the given device extensions and layers to pointers
        let device_extensions: Vec<CString> = device_extensions.iter().map(|extension| to_cstring!(extension)).collect();
        let device_layers: Vec<CString>     = device_layers.iter().map(|layer| to_cstring!(layer)).collect();
        let p_device_extensions: Vec<*const i8> = (0..device_extensions.len()).map(|i| device_extensions[i].as_ptr()).collect();
        let p_device_layers: Vec<*const i8>     = (0..device_layers.len()).map(|i| device_layers[i].as_ptr()).collect();

        // Iterate over all physical devices
        let physical_devices = match unsafe { instance.enumerate_physical_devices() } {
            Ok(devices) => devices,
            Err(err)    => { return Err(Error::PhysicalDeviceEnumerateError{ err }); }  
        };
        let mut best_device: Option<(usize, u32)> = None;
        for (i, physical_device) in physical_devices.iter().enumerate() {
            // Get the properties of this device
            let device_properties = unsafe { instance.get_physical_device_properties(*physical_device) };

            // Get a readable name and type
            let device_name: String = match unsafe { CStr::from_ptr(device_properties.device_name.as_ptr()) }.to_str() {
                Ok(name) => name.to_string(),
                Err(err) => { return Err(Error::PhysicalDeviceNameError{ index: i, err }); }
            };

            // Check if this device is supported
            let vk_device_features: vk::PhysicalDeviceFeatures = device_features.into();
            if supports(&instance, *physical_device, i, &device_name, &p_device_extensions, &p_device_layers, &vk_device_features).is_err() { continue; }

            // Select it as best if the first or has a better CPU disconnectedness score
            let device_ranking = DeviceKind::from(device_properties.device_type).score();
            if best_device.is_none() || (device_ranking > best_device.as_ref().unwrap().1) {
                best_device = Some((i, device_ranking));
            }
        }

        // If there is none, error
        match best_device {
            Some((index, _)) => Ok(index),
            None             => Err(Error::NoSupportedPhysicalDevices),
        }
    }

    /// Lists all GPUs that Vulkan can find and that support the given extensions to stdout.
    /// 
    /// # Arguments
    /// - `instance`: The Instance object to seRch for GPUs in.
    /// - `device_extensions`: A slice of extensions that the GPU should support to be marked as 'supported'.
    /// - `device_layers`: A slice of layers that the GPU should support to be marked as 'supported'.
    /// - `device_features`: A struct of features that the GPU should support to be marked as 'supported'.
    /// 
    /// # Returns
    /// Two vectors of (index, name, kind) tuples describing each GPU on success (0 = supported, 1 = unsupported), or else an Error describing the failure on a failure.
    pub fn list(instance: Rc<Instance>, device_extensions: &[&str], device_layers: &[&str], device_features: &DeviceFeatures) -> Result<(Vec<DeviceInfo>, Vec<DeviceInfo>), Error> {
        // Map the given device extensions and layers to pointers
        let device_extensions: Vec<CString> = device_extensions.iter().map(|extension| to_cstring!(extension)).collect();
        let device_layers: Vec<CString>     = device_layers.iter().map(|layer| to_cstring!(layer)).collect();
        let p_device_extensions: Vec<*const i8> = (0..device_extensions.len()).map(|i| device_extensions[i].as_ptr()).collect();
        let p_device_layers: Vec<*const i8>     = (0..device_layers.len()).map(|i| device_layers[i].as_ptr()).collect();

        // Iterate over all physical devices
        let physical_devices = match unsafe { instance.enumerate_physical_devices() } {
            Ok(devices) => devices,
            Err(err)    => { return Err(Error::PhysicalDeviceEnumerateError{ err }); }  
        };
        let mut supported_devices: Vec<DeviceInfo>   = Vec::with_capacity(physical_devices.len());
        let mut unsupported_devices: Vec<DeviceInfo> = Vec::with_capacity(physical_devices.len());
        for (i, physical_device) in physical_devices.iter().enumerate() {
            // Get the properties of this device
            let device_properties = unsafe { instance.get_physical_device_properties(*physical_device) };

            // Get a readable name and cast the type
            let device_name: String = match unsafe { CStr::from_ptr(device_properties.device_name.as_ptr()) }.to_str() {
                Ok(name) => name.to_string(),
                Err(err) => { return Err(Error::PhysicalDeviceNameError{ index: i, err }); }
            };
            let device_type = DeviceKind::from(device_properties.device_type);

            // Get the memory properties
            let device_mem_props: vk::PhysicalDeviceMemoryProperties = unsafe { instance.get_physical_device_memory_properties(*physical_device) };

            // Determine to which list to add it
            let vk_device_features: vk::PhysicalDeviceFeatures = device_features.into();
            if supports(&instance, *physical_device, i, &device_name, &p_device_extensions, &p_device_layers, &vk_device_features).is_ok() {
                supported_devices.push(DeviceInfo {
                    index : i,
                    name  : device_name,
                    kind  : device_type,

                    mem_props : device_mem_props.into(),
                });
            } else {
                unsupported_devices.push(DeviceInfo {
                    index : i,
                    name  : device_name,
                    kind  : device_type,

                    mem_props : device_mem_props.into(),
                });
            }
        }

        // Done! Returns the lists
        Ok((supported_devices, unsupported_devices))
    }



    /// Returns a (cached) list of physical device properties.
    #[inline]
    pub fn get_physical_device_props(&self) -> &PhysicalDeviceProperties { &self.props }

    /// Returns the list of supported features for the given Surface.
    /// 
    /// # Arguments
    /// - `surface`: The Surface to check this device's support of.
    /// 
    /// # Returns
    /// A SwapchainSupport struct detailing the supported capabilities, formats and present modes. If an error occurred, returns an Error instead.
    /// 
    /// # Errors
    /// This function may error when the device could not be queried for its support or the surface is not supported at all.
    pub fn get_swapchain_support(&self, surface: &Rc<Surface>) -> Result<SwapchainSupport, Error> {
        // Check if the chosen graphics queue can present to the given chain
        if !match unsafe {
            surface.get_physical_device_surface_support(self.physical_device, self.families.graphics, surface.vk())
        } {
            Ok(supports) => supports,
            Err(err)     => { return Err(Error::SurfaceSupportError{ err }); }
        } {
            return Err(Error::UnsupportedSurface);
        }

        // Get the surface capabilities
        let capabilities = match unsafe {
            surface.get_physical_device_surface_capabilities(self.physical_device, surface.vk())
        } {
            Ok(capabilities) => capabilities,
            Err(err)         => { return Err(Error::SurfaceCapabilitiesError{ err }); }
        };

        // Get the surface formats
        let formats = match unsafe {
            surface.get_physical_device_surface_formats(self.physical_device, surface.vk())
        } {
            Ok(formats) => formats,
            Err(err)    => { return Err(Error::SurfaceFormatsError{ err }); }
        };

        // Get the surface capabilities
        let present_modes = match unsafe {
            surface.get_physical_device_surface_present_modes(self.physical_device, surface.vk())
        } {
            Ok(present_modes) => present_modes,
            Err(err)          => { return Err(Error::SurfacePresentModesError{ err }); }
        };

        

        // If no formats and present modes are found, we call it not supported at all
        if formats.is_empty() && present_modes.is_empty() { return Err(Error::UnsupportedSurface); }

        // Otherwise, return the new swapchain support!
        Ok(SwapchainSupport {
            capabilities,
            formats,
            present_modes,
        })
    }



    /// Returns the instance around which this Device is wrapped
    #[inline]
    pub fn instance(&self) -> &Rc<Instance> { &self.instance }    

    /// Returns the internal device.
    #[inline]
    pub fn ash(&self) -> &ash::Device { &self.device }

    /// Returns the internal physical device.
    #[inline]
    pub fn physical_device(&self) -> vk::PhysicalDevice { self.physical_device }

    /// Returns the internal Queues struct, which contains the queues used on this device.
    #[inline]
    pub fn queues(&self) -> &Queues { &self.queues }



    /// Returns the index of this device.
    #[inline]
    pub fn index(&self) -> usize { self.index }

    /// Returns the name of this device.
    #[inline]
    pub fn name(&self) -> &str { &self.props.name }
    
    /// Returns the type of this device.
    #[inline]
    pub fn kind(&self) -> DeviceKind { self.props.kind }
    
    /// Returns information about the QueueFamilies for this device.
    #[inline]
    pub fn families(&self) -> &QueueFamilyInfo { &self.families }
}

impl Drop for Device {
    fn drop(&mut self) {
        // Destroy the internal device
        log_destroy!(self, Device);
        unsafe { self.device.destroy_device(None); };
    }
}

impl Deref for Device {
    type Target = ash::Device;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
