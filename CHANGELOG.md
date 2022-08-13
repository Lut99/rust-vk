# Changelog
This file will maintain a list of changes per release of the rust-vk crate.


## [4.0.0] - 2022-08-13
### Added
- `Device` now caches the device properties on creation, which may be returned using `Device::get_physical_device_props()`.
- Auxillary `PhysicalDeviceProperties` struct and related ones (`PhysicalDeviceLimits` and `PhysicalDeviceSparseProperties`).

### Changed
- `Device::kind()` now returns a direct DeviceKind instead of a reference. **[breaking]**


## [3.0.1] - 2022-08-13
### Added
- `vk_attributes` to the `Vertex` struct.

### Fixed
- `StagingBuffer::new_for()` accepting a non-Rc reference (which was incompatible with the Rc-style buffers).


## [3.0.0] - 2022-08-11
### Added
- The new `IndexType` auxillary type.
- The new `IndexBuffer` buffer type.
- `StagingBuffer` now accepts another Buffer to initialize itself for with the `StagingBuffer::new_for` constructor.
- `CommandBuffer::bind_index_buffer()` to the `CommandBuffer` struct.
- `CommandBuffer::draw_indexed()` to the `CommandBuffer` struct.

## Changed
- `VertexBuffer` now accepts a buffer type and a number of vertices instead of a total capacity. **[breaking]**
- All of the buffer's constructors (i.e., that of `StagingBuffer` and `VertexBuffer`) to choose a default value of `SharingMode::Exclusive` for the sharing_mode. Instead, a new constructor, `::new_with_sharing_mode`, that takes the SharingMode too.


## [2.0.2] - 2022-08-11
### Added
- `PartialEq` and `Eq` to `Extent2D`, `Offset2D` and `Rect2D`.


## [2.0.1] - 2022-08-09
### Added
- `From<winit::dpi::PhysicalPosition>` implementation for `Offset2D` and vice versa.
- `From<winit::dpi::PhysicalSize>` implementation for `Extent2D` and vice versa.


## [2.0.0] - 2022-08-06
### Changed
- Return type of `Surface` to `Rc<RefCell<...>>` instead of `Arc<RwLock<...>>`. **[breaking]**


## [1.0.1] - 2022-08-06
### Fixed
- Features not working (used `features` instead of `feature` in `#cfg[]`).
- Incorrect date in CHANGELOG.md for version 1.0.0.


## [1.0.0] - 2022-08-06
### Added
- Initial set of objects, taken from the [Game-Rust](https://github.com/Lut99/Game-Rust) project.
- A README.md.
- A .gitignore file.

### Changed
- Some dependencies (`log` and `winit`) to be tied to a Cargo feature.
- Cargo.toml to have more information and updated name/version number.
- Return type of `CommandPool` and various `MemoryPool`s to `Rc<RefCell<...>>` instead of `Arc<RwLock<...>>`.
- Decription of each file to use Rust docstrings.
