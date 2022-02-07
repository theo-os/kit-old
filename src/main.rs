use std::{io::Write, path::PathBuf};

use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitConfig {
    pub images: Vec<String>,
    pub folders: Vec<String>,
    pub cmdline: String,
    // the path to the kernel bzImage
    pub kernel: String,
}

impl KitConfig {
    pub fn from_file(path: &str) -> Result<KitConfig, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(path)?;
        let config: KitConfig = toml::from_str(&file)?;
        Ok(config)
    }
}

pub async fn build() -> Result<()> {
    // Create a new config from config.hjson
    let config = KitConfig::from_file("kit.toml").unwrap();

    // if there is already a build directory, delete it
    let build_dir = PathBuf::from("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).unwrap();
    }

    // Create a build directory
    let build_dir = "./build";
    std::fs::create_dir_all(build_dir).context("Failed to create build directory")?;

    let rootfs_path = PathBuf::from(build_dir).join("rootfs");
    let rootfs_path = rootfs_path.to_str().unwrap();

    std::fs::create_dir_all(rootfs_path).context("Failed to create rootfs directory")?;

    // For each image in the configuration,
    // utilize `podman save` to pull the image into a tar file
    // and extract it using undocker into the rootfs
    for image in config.images {
        let image_id = image.replace("/", "_").replace(":", "_");

        debug!("Building image {}", image);

        let cmd = format!("podman image save --format=docker-archive -o {image_id}.tar {image}");
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(build_dir)
            .stderr(std::process::Stdio::inherit())
            .output()
            .context("Failed to save image using podman")?;
        debug!("{}", String::from_utf8_lossy(&output.stdout));

        // sh -c undocker image_id.tar - | tar -xvf - -C rootfs
        debug!("Extracting image {}", image);
        let cmd = format!(
            "undocker {}/{}.tar - | tar -xvf - -C {}",
            build_dir, image_id, rootfs_path
        );

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stderr(std::process::Stdio::inherit())
            .output()
            .context("Failed to execute undocker")?;

        debug!("{}", String::from_utf8_lossy(&output.stdout));
    }

    for folder in config.folders {
        debug!("Copying folder {}", folder);
        let cmd = format!("cp -r {}/* {}", folder, rootfs_path);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stderr(std::process::Stdio::inherit())
            .output()
            .context("Failed to execute cp")?;
        debug!("{}", String::from_utf8_lossy(&output.stdout));
    }

    // Next, build the initramfs using find and cpio
    // let initramfs_path = PathBuf::from(build_dir).join("initramfs");
    // let initramfs_path = initramfs_path.to_str().unwrap();

    // let cmd = format!(
    //     "find {} -print0 | cpio --null -ov --format=newc | gzip -9 > {}",
    //     initramfs_path, initramfs_path
    // );

    // let output = std::process::Command::new("sh")
    //     .arg("-c")
    //     .arg(cmd)
    //     .current_dir(build_dir)
    //     .output()
    //     .context("failed to build the initramfs image")?;

    // println!("{}", String::from_utf8_lossy(&output.stdout));

    // Next, create a grub.cfg file in the rootfs
    let grub_path = PathBuf::from(build_dir).join("grub.cfg");
    let grub_path = grub_path.to_str().unwrap();

    let mut file = std::fs::File::create(grub_path).unwrap();
    file.write_all(
        format!(
            "set default=0\n\
            set timeout=5\n\
            menuentry \"Linux\" {{\n\
            linux {} {}\n\
            }}\n",
            config.kernel, config.cmdline
        )
        .as_bytes(),
    )?;

    // Create a new directory in the build directory to hold the final image
    let image_path = PathBuf::from(build_dir).join("iso");
    let image_path = image_path.to_str().unwrap();

    std::fs::create_dir_all(image_path).context("Failed to create image directory")?;

    // create boot/grub in iso/
    let grub_path = PathBuf::from(image_path).join("boot").join("grub");
    let grub_path = grub_path.to_str().unwrap();

    std::fs::create_dir_all(grub_path).context("Failed to create grub directory")?;

    // create rootfs/{etc,sys,proc,run,tmp,dev} if they don't exist
    let paths = vec!["etc", "sys", "proc", "run", "tmp", "dev", "oldroot"];
    for path in paths {
        let path = PathBuf::from(rootfs_path).join(path);
        if !path.exists() {
            std::fs::create_dir_all(&path).context("Failed to create directory")?;
        }
    }

    // copy the rootfs into the image directory
    let cmd = format!("cp -r {}/* {}", rootfs_path, image_path);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to copy rootfs")?;

    debug!("{}", String::from_utf8_lossy(&output.stdout));

    // copy the grub.cfg into the image directory
    let cmd = format!("cp {} {}", build_dir.to_string() + "/grub.cfg", grub_path);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("Failed to copy grub.cfg")?;

    debug!("{}", String::from_utf8_lossy(&output.stdout));

    // Create the image and write it to build/os.iso
    debug!("Creating final image");

    let cmd = format!("grub-mkrescue -o {}/os.iso {}", build_dir, image_path);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("failed to create the iso")?;

    debug!("{}", String::from_utf8_lossy(&output.stdout));

    // let the user know where the build directory is
    info!(
        "A bootable disk image has been placed in: {} as os.iso",
        build_dir
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    if let Err(e) = build().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
