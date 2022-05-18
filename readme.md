# Kit

A linux-kit inspired tool that creates bootable disk images from docker images.

## Requirements

- an existing linux system
- podman
- btrfs-progs
- [rust and cargo](https://rustup.rs)
- [undocker](https://git.sr.ht/~motiejus/undocker)

## Running from Source

```bash
RUST_LOG=debug cargo run
```

## How It Works

- Reads a config file at kit.toml
- For each image specified in **images**, Kit automatically downloads and extracts its contents using skopeo and undocker.
- Writes a limine.cfg file and appends the **cmdline** and **kernel path** (eg. /kernel).

## Example Config

```toml
images = [
    "docker.flowtr.dev/theos/kernel:latest",
    "docker.flowtr.dev/theos/vsh:latest",
    "docker.flowtr.dev/theos/core:latest",
    "docker.flowtr.dev/theos/coreutils:latest",
]
folders = []
kernel = "/kernel/bzImage"
cmdline = "console=tty0 console=ttyS0 console=ttyAMA0 ro
ot=/dev/sda rw init=/bin/theos-init"
boot_protocol = "linux"
```
