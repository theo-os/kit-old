# Kit

A linux-kit inspired tool that creates bootable disk images from docker images.

## Requirements

- podman
- skopeo
- grub
- [rust and cargo](https://rustup.rs)
- [undocker](https://git.sr.ht/~motiejus/undocker)

## Running from Source

```bash
RUST_LOG=debug cargo run
```

## How It Works

- Reads a config file at kit.hjson
- For each image specified in **images**, Kit automatically downloads and extracts its contents using skopeo and undocker.
- Writes a grub.cfg file and appends the **cmdline** and **kernel path** (eg. /kernel).
- grub-mkrescue is used to create the final ISO image.

## Example Config

```json
{
    "images": [
        "docker.flowtr.dev/timdows/kernel:latest",
        "docker.io/busybox:latest"
    ],
    "kernel": "/kernel",
    "cmdline": "console=tty0 console=ttyS0 console=ttyAMA0 root=/dev/sda3 init=/bin/sh"
}
```
