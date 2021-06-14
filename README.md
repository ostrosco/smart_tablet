# smart_tablet

An open source program for smart tablets.

## Requirements

This project depends on a few things:
* Rust - https://www.rust-lang.org/
* Yarn - https://yarnpkg.com/

## Building

To build the application:

```
cargo build --release
```

If you want to cross-compile the application for use on the Raspberry Pi, you'll need to install
`cross` and either `docker` or `podman`. 

```
cargo install cross
```

Then build your image using the provided Dockerfile. `cross` is configured to look locally for
a image named `smart_tablet`.

```
podman build . -t smart_tablet
```

Lastly, cross-compile:

```
cross build --release --target=armv7-unknown-linux-gnueabihf
```
