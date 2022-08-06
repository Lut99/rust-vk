//  INSTANCE.rs
//    by Lut99
// 
//  Created:
//    26 Mar 2022, 14:10:40
//  Last edited:
//    06 Aug 2022, 11:36:30
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains the wrapper around the Vulkan instance.
// 

use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

use ash::vk;
use semver::Version;

use crate::{debug, error, info, warn, to_cstring};
pub use crate::errors::InstanceError as Error;
use crate::log_destroy;


/***** HELPER FUNCTIONS *****/
/// Returns the proper extensions for the target OS' window system.  
/// This overload is for Windows.
/// 
/// **Returns**  
/// The list of required extensions, as a list of CStrings.
#[cfg(all(windows))]
fn os_surface_extensions() -> Vec<CString> {
    // Import the extension into the namespace
    use ash::extensions::khr::Win32Surface;

    // Return the name of the Windows surface extension
    vec![
        Win32Surface::name().to_owned()
    ]
}

/// Returns the proper extensions for the target OS' window system.  
/// This overload is for macOS.
/// 
/// **Returns**  
/// The list of required extensions, as a list of CStrings.
#[cfg(target_os = "macos")]
fn os_surface_extensions() -> Vec<CString> {
    // Import the extension into the namespace
    use ash::extensions::mvk::MacOSSurface;

    // Return the name of the macOS surface extension
    vec![
        MacOSSurface::name().to_owned()
    ]
}

/// Returns the proper extensions for the target OS' window system.  
/// This overload is for Linux (X11).
/// 
/// **Returns**  
/// The list of required extensions, as a list of CStrings.
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
fn os_surface_extensions() -> Vec<CString> {
    // Bring the extensions into the namespace
    use ash::extensions::khr::XlibSurface;
    use ash::extensions::khr::WaylandSurface;

    // Enable both the X11 and the Wayland extension, as, at compile time, we have no idea which should be the one
    vec![
        XlibSurface::name().to_owned(),
        WaylandSurface::name().to_owned(),
    ]
}





/***** POPULATE FUNCTIONS *****/
/// Populates an ApplicationInfo struct.
/// 
/// This function requires that the given CStrings are alive as long as the ApplicationInfo is.
/// 
/// The application version (`version`) and engine version will be converted to a Vulkan version number automatically.
fn populate_app_info<'a>(name: &'a CStr, version: Version, engine: &'a CStr, engine_version: Version) -> vk::ApplicationInfo {
    // Convert the versions to Vulkan versions
    let version        = vk::make_api_version(0, version.major as u32, version.minor as u32, version.patch as u32);
    let engine_version = vk::make_api_version(0, engine_version.major as u32, engine_version.minor as u32, engine_version.patch as u32);

    // Finally, construct the application info
    vk::ApplicationInfo {
        s_type              : vk::StructureType::APPLICATION_INFO,
        p_next              : ptr::null(),
        p_application_name  : name.as_ptr(),
        application_version : version,
        p_engine_name       : engine.as_ptr(),
        engine_version      : engine_version,
        api_version         : vk::API_VERSION_1_1,
    }
}

/// Populates a DebugUtilsMessengerCreateInfoEXT struct.
/// 
/// This function sets 'vulkan_debug_callback' as the callback for the debug create info.
#[inline]
fn populate_debug_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type : vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next : ptr::null(),

        flags             : vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity  :
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type      :
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback : Some(vulkan_debug_callback),
        p_user_data       : ptr::null_mut(),
    }
}

/// Populates an InstanceCreateInfo struct.
/// 
/// This function assumes that the given references will be valid at least through the call to create the Instance.
/// 
/// It will also validate if the given layers and extensions exist in the local Vulkan installation, for which it uses the given Entry.
/// 
/// # Errors
/// 
/// This function will return an Error if we could not query the entry for properties, or if either an extension or a layer does not exist in the local Vulkan installation.
fn populate_instance_info(entry: &ash::Entry, app_info: &vk::ApplicationInfo, debug_info: &Option<vk::DebugUtilsMessengerCreateInfoEXT>, p_extensions: &[*const i8], p_layers: &[*const i8]) -> Result<vk::InstanceCreateInfo, Error> {
    // Define the lists the verify who we already had
    let mut verify_ext: Vec<bool> = p_extensions.iter().map(|_| false).collect();
    let mut verify_lay: Vec<bool> = p_layers.iter().map(|_| false).collect();

    // First check: global extensions
    let global_extensions = match entry.enumerate_instance_extension_properties(None) {
        Ok(layers) => layers,
        Err(err)   => { return Err(Error::ExtensionEnumerateError{ layer: None, err }); }
    };
    for i in 0..p_extensions.len() {
        // Get the required extension
        let req_ext: &CStr = unsafe { &CStr::from_ptr(p_extensions[i]) };

        // Loop through the available extensions to find it
        for avail_ext in &global_extensions {
            // Convert the raw extension name to a CString
            let avail_ext: &CStr = unsafe { &CStr::from_ptr(avail_ext.extension_name.as_ptr()) };

            // Compare them
            if req_ext == avail_ext {
                verify_ext[i] = true;
                break;
            }
        }
    }

    // Second check: layers (and layer extensions)
    let avail_layers = match entry.enumerate_instance_layer_properties() {
        Ok(layers) => layers,
        Err(err)   => { return Err(Error::LayerEnumerateError{ err }); }
    };
    for i in 0..p_layers.len() {
        // Get the required layer
        let req_layer: &CStr = unsafe { &CStr::from_ptr(p_layers[i]) };

        // Loop through the available layers to find it
        for avail_layer in &avail_layers {
            // Convert the raw layer name to a CString
            let avail_layer: &CStr = unsafe { &CStr::from_ptr(avail_layer.layer_name.as_ptr()) };

            // Compare them
            if req_layer == avail_layer {
                verify_lay[i] = true;
                break;
            }
        }

        // Error if we still did not find it
        if !verify_lay[i] { return Err(Error::UnknownLayer{ layer: req_layer.to_owned() }); }

        // If we _did_ find it, also try to resolve extensions for this layer
        let layer_extensions = match entry.enumerate_instance_extension_properties(Some(req_layer)) {
            Ok(layers) => layers,
            Err(err)   => { return Err(Error::ExtensionEnumerateError{ layer: Some(req_layer.to_owned()), err }); }
        };
        for i in 0..p_extensions.len() {
            // Get the required extension
            let req_ext: &CStr = unsafe { &CStr::from_ptr(p_extensions[i]) };

            // Loop through the available extensions to find it
            for avail_ext in &layer_extensions {
                // Convert the raw extension name to a CString
                let avail_ext: &CStr = unsafe { &CStr::from_ptr(avail_ext.extension_name.as_ptr()) };

                // Compare them
                if req_ext == avail_ext {
                    verify_ext[i] = true;
                    break;
                }
            }
        }
    }

    // Final check: make sure all extensions are found
    for i in 0..verify_ext.len() {
        if !verify_ext[i] {
            return Err(Error::UnknownExtension{ extension: unsafe { CStr::from_ptr(p_extensions[i]) }.to_owned() });
        }
    }

    // If on macOS, we want the portability extension
    #[allow(unused_variables)]
    let flags: vk::InstanceCreateFlags = vk::InstanceCreateFlags::empty();
    #[cfg(target_os = "macos")]
    let flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;

    // With everything verified, we can finally put it in the struct and return it
    Ok(vk::InstanceCreateInfo {
        s_type                     : vk::StructureType::INSTANCE_CREATE_INFO,
        p_next                     : if let Some(debug_info) = debug_info {
            debug_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const std::os::raw::c_void
        } else {
            ptr::null()
        },
        flags,
        p_application_info         : app_info as *const vk::ApplicationInfo,
        pp_enabled_extension_names : p_extensions.as_ptr(),
        enabled_extension_count    : p_extensions.len() as u32,
        pp_enabled_layer_names     : p_layers.as_ptr(),
        enabled_layer_count        : p_layers.len() as u32,
    })
}





/***** CALLBACKS *****/
/// Callback for the Vulkan debug messenger.
/// 
/// This function takes the message reported by Vulkan, and passes it to the appropriate macro from the log crate.
/// 
/// This function assumes that it goes right with the log crate and multithreading (if applicable).
unsafe extern "system" fn vulkan_debug_callback(
    message_severity : vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type     : vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data  : *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data     : *mut std::os::raw::c_void,
) -> vk::Bool32 {
    // Match the message type
    #[allow(unused_variables)]
    let kind = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL     => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION  => "[Validation]",
        _                                              => "[Unknown]",
    };

    // Send the message with the proper log macro
    #[allow(unused_variables)]
    let message = CStr::from_ptr((*p_callback_data).p_message);
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => debug!("[Vulkan] {} {:?}", kind, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO    => info!("[Vulkan] {} {:?}", kind, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("[Vulkan] {} {:?}", kind, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR   => error!("[Vulkan] {} {:?}", kind, message),
        _                                              => info!("[Unknown] [Vulkan] {} {:?}", kind, message),
    }

    // Done
    vk::FALSE
}





/***** LIBRARY *****/
/// Represents the Instance in the wrapper, which is the application-global instantiation of Vulkan and other libraries.
pub struct Instance {
    /// The ash entry, that determines how we link to the underlying Vulkan library
    entry : ash::Entry,

    /// The instance object that this struct wraps.
    instance : ash::Instance,
    /// The loader (0) and the messenger (1) for Vulkan's DebugUtils.
    debug_utils : Option<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)>,
}

impl Instance {
    /// Constructor for the Instance.
    /// 
    /// Every Vulkan app needs its own Instance of the driver, which is what this class represents. There should thus be only one of these.
    /// 
    /// # Generic arguments
    /// - `S1`: The &str-like type of the application's name.
    /// - `S2`: The &str-like type of the application's engine's name.
    /// 
    /// # Arguments
    /// - `name`: The name of the application to register in the Vulkan driver.
    /// - `version`: The version of the application to register in the Vulkan driver.
    /// - `engine_name`: The name of the application's engine to register in the Vulkan driver.
    /// - `engine_version`: The version of the application's engine to register in the Vulkan driver.
    /// - `additional_extensions`: A slice of additional extensions to enable in the application-global instance.
    /// - `additional_layers`: A slice of additional validation layers to enable in the application-global instance.
    /// 
    /// # Returns
    /// The new Instance instance on success, or else an Error describing why we failed to create it.
    pub fn new<'a, 'b, S1: AsRef<str>, S2: AsRef<str>>(name: S1, version: Version, engine: S2, engine_version: Version, additional_extensions: &[&'a str], additional_layers: &[&'b str]) -> Result<Rc<Self>, Error> {
        // Convert the str-like into &str
        let name: &str   = name.as_ref();
        let engine: &str = engine.as_ref();



        // Create the entry
        let entry = unsafe {
            match ash::Entry::load() {
                Ok(entry) => entry,
                Err(err)  => { return Err(Error::LoadError{ err }); }
            }
        };



        // Get a CString from the String
        let cname   = to_cstring!(name);
        let cengine = to_cstring!(engine);

        // Construct the ApplicationInfo
        let app_info = populate_app_info(&cname, version, &cengine, engine_version);



        // First, check if we should enable debug
        let debug = additional_layers.contains(&"VK_LAYER_KHRONOS_validation");

        // Convert both list of additional extensions/layers to CStrs
        let mut additional_extensions: Vec<CString> = (0..additional_extensions.len()).map(|i| to_cstring!(additional_extensions[i])).collect();
        let additional_layers: Vec<CString>         = (0..additional_layers.len()).map(|i| to_cstring!(additional_layers[i])).collect();

        // Collect the required extensions
        let mut extensions: Vec<CString> = vec![ ash::extensions::khr::Surface::name().to_owned() ];
        #[cfg(target_os = "macos")]
        { extensions.push(InstanceExtension::PortabilityEnumeration.into()); }
        if debug { extensions.push(ash::extensions::ext::DebugUtils::name().to_owned()); }
        extensions.append(&mut os_surface_extensions());

        // Merge the extensions and the layers
        extensions.append(&mut additional_extensions);
        let layers = additional_layers;



        // If required, instantiate the DebugInfo
        let debug_info: Option<vk::DebugUtilsMessengerCreateInfoEXT> = if debug {
            Some(populate_debug_info())
        } else {
            None
        };



        // Cast the extensions and layers to pointers
        let p_extensions: Vec<*const i8> = (0..extensions.len()).map(|i| extensions[i].as_ptr()).collect();
        let p_layers: Vec<*const i8>     = (0..layers.len()).map(|i| layers[i].as_ptr()).collect();

        // Finally, create the InstanceInfo
        let instance_info: vk::InstanceCreateInfo = populate_instance_info(&entry, &app_info, &debug_info, &p_extensions, &p_layers)?;



        // Use that to create the instance
        let instance: ash::Instance = unsafe {
            debug!("Initializing instance...");
            match entry.create_instance(&instance_info, None) {
                Ok(instance) => instance,
                Err(err)     => { return Err(Error::CreateError{ err }); }
            }
        };



        // If debugging, then also build the debugger
        let debug_utils = if let Some(debug_info) = debug_info {
            debug!("Initializing debug messenger...");

            // Create the new debug function loader
            let debug_loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
            
            // Create the debug instance
            unsafe {
                match debug_loader.create_debug_utils_messenger(&debug_info, None) {
                    Ok(debug_messenger) => Some((debug_loader, debug_messenger)),
                    Err(err)            => { return Err(Error::DebugCreateError{ err }); }
                }
            }
        } else {
            None
        };



        // Finally, create the struct!
        Ok(Rc::new(Self {
            entry,

            instance,
            debug_utils,
        }))
    }



    /// Returns the internal ash Entry.
    #[inline]
    pub fn ash(&self) -> &ash::Entry { &self.entry }

    /// Returns (an immuteable reference to) the internal Vulkan instance.
    #[inline]
    pub fn vk(&self) -> &ash::Instance { &self.instance }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // If present, destroy the debugger
        if let Some((debug_loader, debug_messenger)) = &self.debug_utils {
            log_destroy!(debug_loader, ash::extensions::ext::DebugUtils);
            unsafe {
                debug_loader.destroy_debug_utils_messenger(*debug_messenger, None);
            }
        }

        // Destroy the instance
        log_destroy!(self, Instance);
        unsafe { self.instance.destroy_instance(None); }
    }
}

impl Deref for Instance {
    type Target = ash::Instance;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.instance }
}
