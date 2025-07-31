# Open Music Server

## Build Instructions

### Linux

Dependencies:

- Rust
- Clang
- vcpkg

Install ffmpeg libs for linking

```
vcpkg install ffmpeg:x64-linux
```

Then build

```
cargo build
```

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
