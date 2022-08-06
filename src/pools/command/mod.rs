//  MOD.rs
//    by Lut99
// 
//  Created:
//    05 May 2022, 10:43:25
//  Last edited:
//    06 Aug 2022, 10:51:03
//  Auto updated?
//    Yes
// 
//  Description:
//!   Entrypoint to the CommandBuffers and CommandPools module.
// 

/// Contains the buffer definitions
pub mod buffers;
/// Contains the pool itself
pub mod pool;


// Bring some stuff into the module scope
pub use buffers::CommandBuffer as Buffer;
pub use pool::{Error, CommandPool as Pool};
