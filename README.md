# Open Music Server

## Building from source

### Dependencies:

- Rust
- Clang
- vcpkg
- [cargo-vcpkg](https://github.com/mcgoo/cargo-vcpkg?tab=readme-ov-file#installation)
- [just](https://github.com/casey/just?tab=readme-ov-file#installation)

### Instructions:

Install ffmpeg libs for linking

```
just install-ffmpeg
```

Then build

```
cargo build
```
