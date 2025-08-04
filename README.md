# Open Music Server

## Build Instructions

### Linux

Dependencies:

- Rust
- Clang
- vcpkg

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

Install ffmpeg libs for linking

```
cargo vcpkg build
```

Then build

```
cargo build
```
