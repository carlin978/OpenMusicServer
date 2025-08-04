# Open Music Server

## Build Instructions

### Linux

Dependencies:

- Rust
- Clang
- vcpkg
- [cargo-vcpkg](https://github.com/mcgoo/cargo-vcpkg?tab=readme-ov-file#installation)

Install ffmpeg libs for linking

```
cargo vcpkg build
```

Then build

```
cargo build
```

### Windows

Dependencies:

- Rust
- vcpkg
- [cargo-vcpkg](https://github.com/mcgoo/cargo-vcpkg?tab=readme-ov-file#installation)

Install ffmpeg libs for linking

```
cargo vcpkg build
```

Then build

```
cargo build
```
