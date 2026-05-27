#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Stdio;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InstallConfig {
    disk: String,
    hostname: String,
    username: String,
    locale: String,
    timezone: String,
    keymap: String,
    dm: String,
    gpu: String,
    swap_size: u32,
}

#[derive(Serialize, Clone)]
pub struct LogLine {
    pub text: String,
    pub kind: String,
}

#[derive(Serialize, Clone)]
pub struct ProgressEvent {
    pub stage: String,
    pub percent: u8,
}

fn log(app: &AppHandle, text: &str, kind: &str) {
    let _ = app.emit(
        "installer:log",
        LogLine {
            text: text.to_string(),
            kind: kind.to_string(),
        },
    );
}

fn progress(app: &AppHandle, stage: &str, percent: u8) {
    let _ = app.emit(
        "installer:progress",
        ProgressEvent {
            stage: stage.to_string(),
            percent,
        },
    );
}

fn validate_safe(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("O campo '{}' não pode ser vazio.", field));
    }
    let unsafe_chars = ['$', '`', ';', '&', '|', '\n', '\r', '"', '\'', '\\', '<', '>'];
    if let Some(c) = value.chars().find(|c| unsafe_chars.contains(c)) {
        return Err(format!("Caractere inválido '{}' encontrado em '{}'.", c, field));
    }
    Ok(())
}

fn validate_config(cfg: &InstallConfig) -> Result<(), String> {
    if !cfg.disk.starts_with("/dev/") {
        return Err("Caminho de disco inválido: deve iniciar com /dev/.".to_string());
    }
    if cfg.swap_size > 64 {
        return Err("O tamanho de swap não pode exceder 64 GB.".to_string());
    }
    validate_safe(&cfg.hostname, "hostname")?;
    validate_safe(&cfg.username, "username")?;
    validate_safe(&cfg.locale, "locale")?;
    validate_safe(&cfg.timezone, "timezone")?;
    validate_safe(&cfg.keymap, "keymap")?;
    validate_safe(&cfg.dm, "display manager")?;

    match cfg.dm.as_str() {
        "gdm" | "sddm" | "lightdm" => {}
        other => return Err(format!("Gerenciador de login desconhecido: {}", other)),
    }
    Ok(())
}

async fn run(app: &AppHandle, cmd: &str, args: &[&str]) -> Result<(), String> {
    log(app, &format!("$ {} {}", cmd, args.join(" ")), "info");

    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Falha ao executar '{}': {}", cmd, e))?;

    if let Some(stdout) = child.stdout.take() {
        let app_c = app.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                log(&app_c, &line, "info");
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app_c = app.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                log(&app_c, &line, "error");
            }
        });
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Erro aguardando processo '{}': {}", cmd, e))?;

    if status.success() {
        log(app, &format!("✓ {} finalizado", cmd), "ok");
        Ok(())
    } else {
        Err(format!(
            "O comando '{}' retornou código de erro {}",
            cmd,
            status.code().unwrap_or(-1)
        ))
    }
}

fn partition(disk: &str, num: u8) -> String {
    if disk.contains("nvme") || disk.contains("mmcblk") {
        format!("{}p{}", disk, num)
    } else {
        format!("{}{}", disk, num)
    }
}

fn gpu_packages(gpu: &str) -> &'static [&'static str] {
    match gpu {
        "nvidia" => &["nvidia", "nvidia-utils", "nvidia-settings"],
        "amd"    => &["mesa", "vulkan-radeon", "libva-mesa-driver"],
        "intel"  => &["mesa", "intel-media-driver", "vulkan-intel"],
        _        => &[],
    }
}

fn de_package(dm: &str) -> &'static str {
    match dm {
        "gdm"     => "gnome",
        "sddm"    => "plasma-meta",
        "lightdm" => "xfce4",
        _         => "",
    }
}

#[tauri::command]
fn get_disks() -> Result<Vec<String>, String> {
    let output = std::process::Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME", "-e", "7,11"])
        .output()
        .map_err(|e| format!("lsblk falhou: {}", e))?;

    let disks = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| format!("/dev/{}", l.trim()))
        .collect();

    Ok(disks)
}

#[tauri::command]
async fn start_installation(app: AppHandle, config: InstallConfig) -> Result<String, String> {
    validate_config(&config)?;

    let disk = &config.disk;
    let part_boot = partition(disk, 1);
    let part_root = partition(disk, 2);

    progress(&app, "Sincronizando relógio de rede (NTP)", 5);
    run(&app, "timedatectl", &["set-ntp", "true"]).await?;

    progress(&app, "Criando partições no disco (GPT)", 10);
    run(&app, "sgdisk", &["-Z", disk]).await?;
    run(&app, "sgdisk", &["-n", "1:0:+2048M", "-t", "1:ef00", "-c", "1:EFI", disk]).await?;
    run(&app, "sgdisk", &["-n", "2:0:0", "-t", "2:8300", "-c", "2:ROOT", disk]).await?;

    progress(&app, "Formatando partições (FAT32 e EXT4)", 20);
    run(&app, "mkfs.fat", &["-F32", &part_boot]).await?;
    run(&app, "mkfs.ext4", &["-F", &part_root]).await?;

    progress(&app, "Montando sistemas de arquivos", 28);
    run(&app, "mount", &[&part_root, "/mnt"]).await?;
    run(&app, "mkdir", &["-p", "/mnt/boot"]).await?;
    run(&app, "mount", &[&part_boot, "/mnt/boot"]).await?;

    if config.swap_size > 0 {
        progress(&app, &format!("Criando arquivo SWAP de {}GB", config.swap_size), 33);
        let swap_arg = format!("{}G", config.swap_size);
        run(&app, "fallocate", &["-l", &swap_arg, "/mnt/swapfile"]).await?;
        run(&app, "chmod", &["600", "/mnt/swapfile"]).await?;
        run(&app, "mkswap", &["/mnt/swapfile"]).await?;
    }

    progress(&app, "Filtrando melhores mirrors (Reflector)", 38);
    let _ = run(&app, "reflector", &["--latest", "10", "--sort", "rate", "--save", "/etc/pacman.d/mirrorlist"]).await;

    progress(&app, "Instalando sistema base do Arch (Aguarde)", 45);

    let mut pkgs: Vec<&str> = vec![
        "/mnt", "--needed",
        "base", "linux", "linux-firmware",
        "nano", "git", "zsh", "wget", "curl", "sudo",
        "networkmanager", "grub", "efibootmgr",
        "reflector", "xorg-server",
        &config.dm,
    ];

    let de = de_package(&config.dm);
    if !de.is_empty() { pkgs.push(de); }

    for pkg in gpu_packages(&config.gpu) { pkgs.push(pkg); }

    run(&app, "pacstrap", &pkgs).await?;

    progress(&app, "Gerando fstab", 70);
    run(&app, "sh", &["-c", "genfstab -U /mnt > /mnt/etc/fstab"]).await?;

    if config.swap_size > 0 {
        run(&app, "sh", &["-c", "echo '/swapfile none swap defaults 0 0' >> /mnt/etc/fstab"]).await?;
    }

    progress(&app, "Escrevendo scripts de chroot", 75);

    let script = format!(
        "#!/bin/bash\nset -euo pipefail\n\
        ln -sf /usr/share/zoneinfo/{tz} /etc/localtime\n\
        hwclock --systohc\n\
        sed -i 's/^#\\({locale}\\)/\\1/' /etc/locale.gen\n\
        locale-gen\n\
        echo 'LANG={locale}' > /etc/locale.conf\n\
        echo 'KEYMAP={keymap}' > /etc/vconsole.conf\n\
        echo '{hostname}' > /etc/hostname\n\
        printf '127.0.0.1 localhost\\n::1 localhost\\n127.0.1.1 {hostname}.localdomain {hostname}\\n' > /etc/hosts\n\
        systemctl enable NetworkManager\n\
        systemctl enable {dm}\n\
        grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB\n\
        grub-mkconfig -o /boot/grub/grub.cfg\n\
        useradd -m -G wheel -s /bin/zsh {username}\n\
        echo '{username}:{username}' | chpasswd\n\
        echo '{username} ALL=(ALL:ALL) ALL' > /etc/sudoers.d/{username}\n\
        chmod 440 /etc/sudoers.d/{username}\n",
        tz       = config.timezone,
        locale   = config.locale,
        keymap   = config.keymap,
        hostname = config.hostname,
        dm       = config.dm,
        username = config.username,
    );

    let script_host_path = "/mnt/tmp/archism_setup.sh";
    std::fs::create_dir_all("/mnt/tmp").map_err(|e| format!("Falha ao criar /mnt/tmp: {}", e))?;
    std::fs::write(script_host_path, &script).map_err(|e| format!("Falha ao criar script de chroot: {}", e))?;

    progress(&app, "Executando configurações internas no chroot", 80);
    run(&app, "arch-chroot", &["/mnt", "bash", "/tmp/archism_setup.sh"]).await?;

    std::fs::remove_file(script_host_path).ok();

    progress(&app, "Concluído", 100);
    log(&app, "✓ Arch Linux instalado com sucesso! Pode reiniciar com segurança.", "ok");

    Ok("Sucesso".to_string())
}

#[tauri::command]
async fn reboot_system() -> Result<(), String> {
    std::process::Command::new("reboot")
        .spawn()
        .map_err(|e| format!("Falha ao reiniciar sistema: {}", e))?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_disks,
            start_installation,
            reboot_system,
        ])
        .run(tauri::generate_context!())
        .expect("Erro executando o motor do instalador Tauri");
}