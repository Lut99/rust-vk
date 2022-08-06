# Changelog
This file will maintain a list of changes per release of the game-vk crate.


## [1.0.0] - 2022-05-15
### Added
- Initial set of objects, taken from the [Game-Rust](https://github.com/Lut99/Game-Rust) project.
- A README.md.
- A .gitignore file

### Changed
- Some dependencies (`log` and `winit`) to be tied to a Cargo feature.
- Cargo.toml to have more information and updated name/version number.
- Return type of `CommandPool` and various `MemoryPool`s to `Rc<RefCell<...>>` instead of `Arc<RwLock<...>>`.
- Decription of each file to use Rust docstrings.