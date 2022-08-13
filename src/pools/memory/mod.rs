//  MOD.rs
//    by Lut99
// 
//  Created:
//    25 Jun 2022, 16:16:04
//  Last edited:
//    13 Aug 2022, 12:47:42
//  Auto updated?
//    Yes
// 
//  Description:
//!   Entrypoint to the Buffers, Images and MemoryPool module.
// 

// Declare modules
pub mod spec;
pub mod block;
pub mod pools;
pub mod buffers;

// Define a prelude to import
pub mod prelude {
    pub use super::spec::{Buffer, HostBuffer, LocalBuffer, MemoryPool, TransferBuffer, Vertex};
}

// Bring some stuff into the module scope
pub use buffers::{IndexBuffer, StagingBuffer, VertexBuffer};
pub use spec::{Buffer, HostBuffer, LocalBuffer, MappedMemory, MemoryPool, TransferBuffer};
pub use pools::{Error, BlockPool, LinearPool, MetaPool};
