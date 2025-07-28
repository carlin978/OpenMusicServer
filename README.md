# Open Music Server

## Build Instructions

### Windows

Dependencies:

- Rust
- vcpkg

Install ffmpeg libs for linking

```
vcpkg install ffmpeg:x64-windows-static-md
```

Then build

```
cargo build
```
