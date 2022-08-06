//  SHADER.rs
//    by Lut99
// 
//  Created:
//    19 Apr 2022, 21:21:27
//  Last edited:
//    06 Aug 2022, 11:07:55
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the Shader struct, which wraps around a single
//!   ShaderModule
// 

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use ash::vk;
use rust_embed::EmbeddedFile;

pub use crate::errors::ShaderError as Error;
use crate::log_destroy;
use crate::device::Device;


/***** LIBRARY *****/
/// The Shader struct, which represents a single piece of Shader code in the render system.
pub struct Shader {
    /// The parent Device where the Shader is compiled for/allocated
    device : Rc<Device>,

    /// The Shader module around which we wrap.
    module : vk::ShaderModule,
}

impl Shader {
    /// Constructor for the Shader, which builds it using the given SPIR-V bytecode.
    /// 
    /// # Generic types
    /// - `B`: The byte-slice-like type of the bytecode.
    /// 
    /// # Arguments
    /// - `device`: The Device on which the Shader will live.
    /// - `code`: The Bytecode of the Shader data to compile.
    /// 
    /// # Returns
    /// A new Shader instance on success.
    /// 
    /// # Errors
    /// This function errors if the bytecode is invalid or if the shader module could not be allocated.
    pub fn from_bytes<B: AsRef<[u8]>>(device: Rc<Device>, code: B) -> Result<Rc<Shader>, Error> {
        // Convert the slice-like into a slice
        let code: &[u8] = code.as_ref();

        // Prepare the create info
        let shader_info = vk::ShaderModuleCreateInfo {
            // Do the standard stuff
            s_type : vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next : ptr::null(),  
            flags  : vk::ShaderModuleCreateFlags::empty(),

            // Add the code
            p_code    : code.as_ptr() as *const u32,
            code_size : code.len(),
        };

        // Use that to create a m odule
        let module = unsafe {
            match device.create_shader_module(&shader_info, None) {
                Ok(module) => module,
                Err(err)   => { return Err(Error::ShaderCreateError{ err }); }
            }
        };

        // Create a new instance and return that
        Ok(Rc::new(Self {
            device,
            
            module,
        }))
    }

    /// Constructor for the Shader, which builds it from a SPIR-V file on disk.
    /// 
    /// # Generic types
    /// - `P`: The Path-like type of the (compiled) shader file.
    /// 
    /// # Arguments
    /// - `device`: The Device on which the Shader will live.
    /// - `path`: The path to the SPIR-V shader file.
    /// 
    /// # Returns
    /// A new Shader instance on success.
    /// 
    /// # Errors
    /// This function errors if the file could not be read, the bytecode is invalid or if the shader module could not be allocated.
    pub fn from_path<P: AsRef<Path>>(device: Rc<Device>, path: P) -> Result<Rc<Shader>, Error> {
        // Convert the Path-like into a Path
        let path: &Path = path.as_ref();

        // Load the file as raw bytes
        let handle = match File::open(path) {
            Ok(handle) => handle,
            Err(err)   => { return Err(Error::FileOpenError{ path: path.to_path_buf(), err }); }
        };

        // Read everything into a bytes buffer
        let mut bytes: Vec<u8> = Vec::with_capacity(fs::metadata(path).unwrap_or_else(|err| panic!("Opened file '{}', but could not read size: {}; this should never happen!", path.display(), err)).len() as usize);
        for byte in handle.bytes() {
            // Unwrap the byte
            let byte = match byte {
                Ok(byte) => byte,
                Err(err) => { return Err(Error::FileReadError{ path: path.to_path_buf(), err }); }
            };

            // Add to the list
            bytes.push(byte);
        }

        // With the bytes collected, use from_bytes() to do the actual shader builder
        Self::from_bytes(device, bytes)
    }

    /// Constructor for the Shader, which builds it from embedded SPIR-V code.
    /// 
    /// # Arguments
    /// - `device`: The Device on which the Shader will live.
    /// - `data`: The EmbeddedFile struct that contains the data.
    /// 
    /// # Returns
    /// A new Shader instance on success.
    /// 
    /// # Errors
    /// This function errors if the bytecode is invalid or if the shader module could not be created in the Vulkan backend.
    pub fn from_embedded(device: Rc<Device>, data: EmbeddedFile) -> Result<Rc<Shader>, Error> {
        // Get the data
        let data: &[u8] = data.data.as_ref();

        // Pass it to the other function
        Self::from_bytes(device, data)
    }

    /// Constructor for the Shader, which builds it from embedded SPIR-V code.
    /// 
    /// This overload takes a Result of an EmbeddedFile load rather than the EmbeddedFile directly.
    /// 
    /// # Arguments
    /// - `device`: The Device on which the Shader will live.
    /// - `result`: The Option<EmbeddedFile> struct that contains the data.
    /// 
    /// # Returns
    /// A new Shader instance on success.
    /// 
    /// # Errors
    /// This function errors if the given result is a failure, the bytecode is invalid or if the shader module could not be created in the Vulkan backend.
    pub fn try_embedded(device: Rc<Device>, result: Option<EmbeddedFile>) -> Result<Rc<Shader>, Error> {
        // Unpack the data
        let data = match result {
            Some(data) => data,
            None       => { return Err(Error::EmbeddedError); }
        };

        // Pass it to the other function
        Self::from_embedded(device, data)
    }



    /// Returns the device where the Shader lives.
    #[inline]
    pub fn device(&self) -> &Rc<Device> { &self.device }
    
    /// Returns the Vulkan VkShaderModule around which this struct wraps.
    #[inline]
    pub fn vk(&self) -> vk::ShaderModule { self.module }
}

impl Drop for Shader {
    fn drop(&mut self) {
        log_destroy!(self, Shader);
        unsafe { self.device.destroy_shader_module(self.module, None); }
    }
}
