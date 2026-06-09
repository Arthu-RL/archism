use std::{
    process::Stdio, sync::{Arc, Mutex}
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use config::{InstallConfig, validate_config};
use state::{AppState, Step};

pub fn partition(disk: &str, num: u8) -> String {
    if disk.contains("nvme") || disk.contains("mmcblk") {
        format!("{}p{}", disk, num)
    } else {
        format!("{}{}", disk, num)
    }
}

pub fn gpu_packages(gpu: &str) -> &'static [&'static str] {
    match gpu {
        "nvidia" => &["nvidia-open", "nvidia-utils", "nvidia-settings"],
        "amd"    => &["mesa", "vulkan-radeon", "libva-mesa-driver"],
        "intel"  => &["mesa", "intel-media-driver", "vulkan-intel"],
        _        => &[],
    }
}

pub fn de_package(dm: &str) -> &'static str {
    match dm {
        "gdm" => "gnome",
        "sddm" => "plasma-meta",
        "lightdm" => "xfce4",
        _ => "",
    }
}

pub async fn run_cmd(state: Arc<Mutex<AppState>>, cmd: &str, args: &[&str]) -> Result<(), String> {
    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.logs.push(format!("$ {} {}", cmd, args.join(" ")));
    }

    let mut child: tokio::process::Child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e: std::io::Error| format!("Falha ao executar '{}': {}", cmd, e))?;

    if let Some(stdout) = child.stdout.take() {
        let state_c: Arc<Mutex<AppState>> = state.clone();
        tokio::spawn(async move {
            let mut lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>> = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut s: std::sync::MutexGuard<'_, AppState> = state_c.lock().unwrap();
                s.logs.push(line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let state_c: Arc<Mutex<AppState>> = state.clone();
        tokio::spawn(async move {
            let mut lines: tokio::io::Lines<BufReader<tokio::process::ChildStderr>> = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut s: std::sync::MutexGuard<'_, AppState> = state_c.lock().unwrap();
                s.logs.push(format!("[ERROR] {}", line));
            }
        });
    }

    let status: std::process::ExitStatus = child
        .wait()
        .await
        .map_err(|e: std::io::Error| format!("Erro aguardando processo '{}': {}", cmd, e))?;

    if status.success() {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.logs.push(format!("✓ {} finalizado", cmd));
        Ok(())
    } else {
        Err(format!(
            "O comando '{}' retornou código de erro {}",
            cmd,
            status.code().unwrap_or(-1)
        ))
    }
}

pub async fn perform_installation(state: Arc<Mutex<AppState>>, config: InstallConfig) -> Result<(), String> {
    validate_config(&config)?;

    let disk: &String = &config.disk;
    let part_boot: String = partition(disk, 1);
    let part_root: String = partition(disk, 2);

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Sincronizando relógio (NTP)".to_string();
        s.progress_percent = 5;
    }
    run_cmd(state.clone(), "timedatectl", &["set-ntp", "true"]).await?;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Criando partições GPT".to_string();
        s.progress_percent = 10;
    }
    run_cmd(state.clone(), "sgdisk", &["-Z", disk]).await?;
    run_cmd(state.clone(), "sgdisk", &["-n", "1:0:+2048M", "-t", "1:ef00", "-c", "1:EFI", disk]).await?;
    run_cmd(state.clone(), "sgdisk", &["-n", "2:0:0", "-t", "2:8300", "-c", "2:ROOT", disk]).await?;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Formatando partições".to_string();
        s.progress_percent = 20;
    }
    run_cmd(state.clone(), "mkfs.fat", &["-F32", &part_boot]).await?;
    run_cmd(state.clone(), "mkfs.ext4", &["-F", &part_root]).await?;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Montando sistemas de arquivos".to_string();
        s.progress_percent = 30;
    }
    run_cmd(state.clone(), "mount", &[&part_root, "/mnt"]).await?;
    run_cmd(state.clone(), "mkdir", &["-p", "/mnt/boot"]).await?;
    run_cmd(state.clone(), "mount", &[&part_boot, "/mnt/boot"]).await?;

    if config.swap_size > 0 {
        {
            let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
            s.progress_stage = format!("Criando arquivo SWAP de {}GB", config.swap_size);
            s.progress_percent = 35;
        }
        let swap_arg: String = format!("{}G", config.swap_size);
        run_cmd(state.clone(), "fallocate", &["-l", &swap_arg, "/mnt/swapfile"]).await?;
        run_cmd(state.clone(), "chmod", &["600", "/mnt/swapfile"]).await?;
        run_cmd(state.clone(), "mkswap", &["/mnt/swapfile"]).await?;
    }

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Configurando espelhos rápidos (HTTPS)".to_string();
        s.progress_percent = 40;
    }
    let _ = run_cmd(state.clone(), "reflector", &["--latest", "5", "--protocol", "https", "--sort", "rate", "--save", "/etc/pacman.d/mirrorlist"]).await;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Atualizando chaves de segurança (Keyring)".to_string();
        s.progress_percent = 45;
    }
    run_cmd(state.clone(), "pacman", &["-Sy", "archlinux-keyring", "--noconfirm"]).await?;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Instalando sistema base (Aguarde)".to_string();
        s.progress_percent = 50;
    }

    let mut pkgs: Vec<&str> = vec![
        "/mnt", "--needed", "--noconfirm",
        "base", "linux", "linux-firmware",
        "nano", "git", "wget", "curl", "sudo",
        "networkmanager", "grub", "efibootmgr",
        "xorg-server",
        &config.dm,
    ];

    let de: &str = de_package(&config.dm);
    if !de.is_empty() { pkgs.push(de); }

    for pkg in gpu_packages(&config.gpu) { pkgs.push(pkg); }

    run_cmd(state.clone(), "pacstrap", &pkgs).await?;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Gerando tabela fstab".to_string();
        s.progress_percent = 75;
    }
    run_cmd(state.clone(), "sh", &["-c", "genfstab -U /mnt > /mnt/etc/fstab"]).await?;

    if config.swap_size > 0 {
        run_cmd(state.clone(), "sh", &["-c", "echo '/swapfile none swap defaults 0 0' >> /mnt/etc/fstab"]).await?;
    }

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Escrevendo scripts chroot".to_string();
        s.progress_percent = 80;
    }

    let script: String = format!(
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
        useradd -m -G wheel -s /bin/bash {username}\n\
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

    let script_host_path: &str = "/mnt/archism_setup.sh";
    std::fs::write(script_host_path, &script).map_err(|e: std::io::Error| format!("Falha ao criar script de chroot: {}", e))?;

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Executando configurações no chroot".to_string();
        s.progress_percent = 90;
    }

    run_cmd(state.clone(), "arch-chroot", &["/mnt", "bash", "/archism_setup.sh"]).await?;

    std::fs::remove_file(script_host_path).ok();

    {
        let mut s: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
        s.progress_stage = "Instalação Concluída".to_string();
        s.progress_percent = 100;
        std::thread::sleep(Duration::from_millis(5000));
        s.step = Step::Success;
    }
    Ok(())
}