//  SURFACE.rs
//    by Lut99
// 
//  Created:
//    01 Apr 2022, 17:26:26
//  Last edited:
//    06 Aug 2022, 16:06:04
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the Vulkan Surface wrapper.
// 

use std::ops::Deref;
use std::rc::Rc;

#[allow(unused_imports)]
use ash::{Entry as VkEntry, Instance as VkInstance};
use ash::extensions::khr;
use ash::vk::SurfaceKHR;

#[cfg(feature = "winit")]
use winit::window::Window as WWindow;

pub use crate::errors::SurfaceError as Error;
use crate::log_destroy;
use crate::instance::Instance;


/***** HELPER FUNCTIONS *****/
/// Returns a new surface from the given window.
/// 
/// There are three overloads for this function, each for the target platform. This overload is for Windows.
/// 
/// # Arguments
/// - `entry`: The ash entry struct that is used to load new pointers from.
/// - `instance`: The Vulkan instance that is used to create the new Surface in.
/// - `wwindow`: The winit Window to create the Surface from.
///
/// # Returns
/// A new SurfaceKHR struct.
/// 
/// # Errors
/// This function errors whenever the underlying APIs error.
#[cfg(feature = "winit")]
#[cfg(all(windows))]
unsafe fn create_surface(entry: &VkEntry, instance: &VkInstance, wwindow: &WWindow) -> Result<SurfaceKHR, Error> {
    use std::os::raw::c_void;
    use std::ptr;

    use ash::vk;
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winit::platform::windows::WindowExtWindows;

    use crate::debug;

    
    // Get a Windows Window Handle
    let hwnd = wwindow.hwnd() as HWND;
    // Get the instance handle for this process, which is Window's container of this process' windows
    let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;

    // Now create the appropriate create info
    let surface_info = vk::Win32SurfaceCreateInfoKHR {
        // Set the standard fields
        s_type : vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next : ptr::null(),
        flags  : Default::default(),

        // Pass the instance and the window handle
        hinstance,
        hwnd : hwnd as *const c_void,
    };

    // Build the loader for the surface
    debug!("Creating Windows surface...");
    let loader = khr::Win32Surface::new(entry, instance);
    // Create the new surface
    match loader.create_win32_surface(&surface_info, None) {
        Ok(surface) => Ok(surface),
        Err(err)    => { return Err(Error::WindowsSurfaceKHRCreateError{ err }); }
    }
}

/// Returns a new surface from the given window.
/// 
/// There are three overloads for this function, each for the target platform. This overload is for macOS.
/// 
/// # Arguments
/// - `entry`: The ash entry struct that is used to load new pointers from.
/// - `instance`: The Vulkan instance that is used to create the new Surface in.
/// - `wwindow`: The winit Window to create the Surface from.
///
/// # Returns
/// A new SurfaceKHR struct.
/// 
/// # Errors
/// This function errors whenever the underlying APIs error.
#[cfg(feature = "winit")]
#[cfg(target_os = "macos")]
unsafe fn create_surface(entry: &VkEntry, instance: &VkInstance, wwindow: &WWindow) -> Result<SurfaceKHR, Error> {
    use std::mem;
    use std::os::raw::c_void;
    use std::ptr;

    use ash::extensions::mvk::MacOSSurface;
    use ash::vk;
    use cocoa::appkit::{NSView, NSWindow};
    use cocoa::base::id as cocoa_id;
    use metal::MetalLayer;
    use objc::runtime::YES;
    use winit::platform::macos::WindowExtMacOS;

    use crate::debug;

    
    // Get the ID of the window
    let window: cocoa_id = mem::transmute(wwindow.ns_window());

    // Create an as-blank-as-possible animation layer to redner to
    let layer: MetalLayer = MetalLayer::new();
    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();

    // Get the window's view, and put the animation layer there
    let view: cocoa_id = window.contentView();
    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(mem::transmute(layer.as_ref()));
    view.setWantsLayer(YES);

    // Now use the view in the create info
    let surface_info = vk::MacOSSurfaceCreateInfoMVK {
        // Set the standard fields
        s_type : vk::StructureType::MACOS_SURFACE_CREATE_INFO_MVK,
        p_next : ptr::null(),
        flags  : Default::default(),

        // Pass the view to create the surface on
        p_view : wwindow.ns_view() as *const c_void,
    };

    // Create the surface!
    debug!("Creating macOS Cocoa surface...");
    let loader = MacOSSurface::new(entry, instance);
    // Create the new surface
    match loader.create_mac_os_surface(&surface_info, None) {
        Ok(surface) => Ok(surface),
        Err(err)    => { return Err(Error::MacOSSurfaceKHRCreateError{ err }); }
    }
}

/// Returns a new surface from the given window.
/// 
/// There are three overloads for this function, each for the target platform. This overload is for linux (X11).
/// 
/// # Arguments
/// - `entry`: The ash entry struct that is used to load new pointers from.
/// - `instance`: The Vulkan instance that is used to create the new Surface in.
/// - `wwindow`: The winit Window to create the Surface from.
///
/// # Returns
/// A new SurfaceKHR struct.
/// 
/// # Errors
/// This function errors whenever the underlying APIs error.
#[cfg(feature = "winit")]
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
unsafe fn create_surface(entry: &VkEntry, instance: &VkInstance, wwindow: &WWindow) -> Result<SurfaceKHR, Error> {
    use std::ptr;

    use ash::vk;
    use winit::platform::unix::WindowExtUnix;

    use crate::debug;


    // First, determine which platform we're on
    if wwindow.xlib_display().is_some() {
        // We're on X11

        // Get the winit window as X11 display & window
        let x11_display = wwindow.xlib_display().expect("We are confirmed on X11, but could not get X11 display; this should never happen!");
        let x11_window  = wwindow.xlib_window().expect("We are confirmed on X11, but could not get X11 window; this should never happen!");

        // Use those to create the create info
        let surface_info = vk::XlibSurfaceCreateInfoKHR {
            // Set the standard fields
            s_type : vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
            p_next : ptr::null(),
            flags  : Default::default(),

            // Pass the window & display
            window : x11_window as vk::Window,
            dpy    : x11_display as *mut vk::Display,
        };

        // Create the Surface with that
        debug!("Creating X11 surface...");
        let loader = khr::XlibSurface::new(entry, instance);
        match loader.create_xlib_surface(&surface_info, None) {
            Ok(surface) => Ok(surface),
            Err(err)    => { return Err(Error::X11SurfaceKHRCreateError{ err }); }
        }

    } else if wwindow.wayland_display().is_some() {
        // We're on Wayland

        // Get the winit window as Wayland surface & display
        let wayland_display = wwindow.wayland_display().expect("We are confirmed on Wayland, but could not get Wayland display; this should never happen!");
        let wayland_surface = wwindow.wayland_surface().expect("We are confirmed on Wayland, but could not get Wayland surface; this should never happen!");

        // Use that to create the create info
        let surface_info = vk::WaylandSurfaceCreateInfoKHR {
            // Set the standard fields
            s_type : vk::StructureType::WAYLAND_SURFACE_CREATE_INFO_KHR,
            p_next : ptr::null(),
            flags  : Default::default(),

            // Pass the surface & display
            surface : wayland_surface,
            display : wayland_display,
        };

        // Create the Surface with that
        debug!("Creating Wayland surface...");
        let loader = khr::WaylandSurface::new(entry, instance);
        match loader.create_wayland_surface(&surface_info, None) {
            Ok(surface) => Ok(surface),
            Err(err)    => { return Err(Error::WaylandSurfaceCreateError{ err }); }
        }

    } else {
        // Unsupported window system
        Err(Error::UnsupportedWindowSystem)
    }
}





/***** LIBRARY *****/
/// Implements a Surface, which can be build from a given Window object.
pub struct Surface {
    /// The Instance that this Surface is build on.
    instance : Rc<Instance>,

    /// The load for the surface which we wrap.
    loader  : khr::Surface,
    /// The SurfaceKHR which we wrap.
    surface : SurfaceKHR,
}

impl Surface {
    /// Constructor for the Surface.
    /// 
    /// This function tries to build a surface from the given winit::Window object.
    /// 
    /// # Arguments
    /// - `instance`: The instance where the new Surface will be allocated.
    /// - `wwindow`: The winit Window to build the Surface around.
    /// 
    /// # Returns
    /// A new Surface object, already wrapped in an Rc.
    /// 
    /// # Errors
    /// This function errors whenever the backend Vulkan errors.
    #[cfg(feature = "winit")]
    pub fn new_winit(instance: Rc<Instance>, wwindow: &WWindow) -> Result<Rc<Self>, Error> {
        use crate::debug;

        // Create the surface KHR
        debug!("Initializing surface...");
        let surface = unsafe { create_surface(instance.ash(), instance.vk(), wwindow) }?;

        // Create the accopmanying loader
        let loader = khr::Surface::new(instance.ash(), instance.vk());

        // Store them internally, done
        Ok(Rc::new(Self {
            instance,

            loader,
            surface,
        }))
    }



    /// Returns the instance of the Surface.
    #[inline]
    pub fn instance(&self) -> &Rc<Instance> { &self.instance }

    /// Returns the internal Surface (loader) object.
    #[inline]
    pub fn ash(&self) -> &khr::Surface { &self.loader }

    /// Returns the internal SurfaceKHR object.
    #[inline]
    pub fn vk(&self) -> SurfaceKHR { self.surface }
}

impl Drop for Surface {
    fn drop(&mut self) {
        // Destroy the surface using the loader
        log_destroy!(self, Surface);
        unsafe { self.loader.destroy_surface(self.surface, None); }
    }
}

impl Deref for Surface {
    type Target = khr::Surface;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.loader
    }
}
