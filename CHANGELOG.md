# Changelog
This file will maintain a list of changes per release of the rust-vk crate.


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
