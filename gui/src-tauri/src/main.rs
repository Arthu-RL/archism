// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct InstallConfig {
    disk: String,
    hostname: String,
    username: String,
    locale: String,
    timezone: String,
    keymap: String,
    dm: String,
    gpu: String,
}

// Helper to run shell commands safely
fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
fn get_disks() -> Result<Vec<String>, String> {
    // Modernized disk fetching using lsblk instead of relying on /dev/sda hardcoding
    let out = run_cmd("lsblk", &["-d", "-n", "-o", "NAME", "-e", "7,11"])?;
    let disks = out.lines().map(|l| format!("/dev/{}", l.trim())).collect();
    Ok(disks)
}

#[tauri::command]
async fn start_installation(config: InstallConfig) -> Result<String, String> {
    // 1. Time Sync
    run_cmd("timedatectl", &["set-ntp", "true"])?;

    // 2. Partitioning (Ruthlessly optimized sgdisk calls)
    let disk = &config.disk;
    run_cmd("sgdisk", &["-Z", disk])?;
    run_cmd("sgdisk", &["-n", "1:0:+2048M", "-t", "1:ef00", "-c", "1:EFI", disk])?;
    run_cmd("sgdisk", &["-n", "2:0:0", "-t", "2:8300", "-c", "2:ROOT", disk])?;

    // Determine partitions dynamically (handles NVMe e.g., nvme0n1p1 vs sda1)
    let part_boot = format!("{}1", if disk.contains("nvme") { format!("{}p", disk) } else { disk.to_string() });
    let part_root = format!("{}2", if disk.contains("nvme") { format!("{}p", disk) } else { disk.to_string() });

    // 3. Formatting & Mounting
    run_cmd("mkfs.fat", &["-F32", &part_boot])?;
    run_cmd("mkfs.ext4", &["-F", &part_root])?;
    run_cmd("mount", &[&part_root, "/mnt"])?;
    run_cmd("mkdir", &["-p", "/mnt/boot"])?;
    run_cmd("mount", &[&part_boot, "/mnt/boot"])?;

    // 4. Base Install (Combined package list to eliminate redundant pacman calls)
    let mut base_pkgs = vec![
        "/mnt", "--needed", "base", "linux", "linux-firmware", "nano", 
        "git", "zsh", "wget", "curl", "sudo", "networkmanager", 
        "grub", "efibootmgr", "reflector", "xorg", &config.dm,
    ];
    run_cmd("pacstrap", &base_pkgs)?;
    run_cmd("sh", &["-c", "genfstab -U /mnt >> /mnt/etc/fstab"])?;

    // 5. System Configuration (via arch-chroot)
    // We pass a heredoc into arch-chroot to execute the internal setup directly, 
    // eliminating the need to download a stage 2 script.
    let chroot_script = format!(
        r#"
        ln -sf /usr/share/zoneinfo/{tz} /etc/localtime
        hwclock --systohc
        sed -i 's/^#\({locale}\)/\1/' /etc/locale.gen
        locale-gen
        echo "LANG={locale}" > /etc/locale.conf
        echo "KEYMAP={keymap}" > /etc/vconsole.conf
        echo "{hostname}" > /etc/hostname
        systemctl enable NetworkManager {dm}
        grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
        grub-mkconfig -o /boot/grub/grub.cfg
        "#,
        tz = config.timezone,
        locale = config.locale,
        keymap = config.keymap,
        hostname = config.hostname,
        dm = config.dm
    );

    run_cmd("arch-chroot", &["/mnt", "bash", "-c", &chroot_script])?;

    Ok("Installation completed successfully.".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_disks, start_installation])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

    tauri_app_lib::run()
}
