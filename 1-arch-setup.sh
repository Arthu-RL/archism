#!/bin/bash
set -euo pipefail

# --- Logging ---
log() { echo -e "\033[1;32m[$(date +'%H:%M:%S')] $1\033[0m"; }

# --- Validate arguments ---
if [ "$#" -ne 8 ]; then
    echo "Usage: $0 <username> <password> <hostname> <locale> <timezone> <display_manager> <keymap> <gpu_vendor>"
    exit 1
fi

USERNAME="$1"
PASSWORD="$2"
HOSTNAME="$3"
LOCALE="$4"
TIMEZONE="$5"
DM="$6"
KEYMAP="$7"
GPU_VENDOR="$8"

# --- Time & Locale ---
log "Setting timezone, locale, hostname..."
ln -sf /usr/share/zoneinfo/$TIMEZONE /etc/localtime
hwclock --systohc
sed -i "s/^#\(${LOCALE}\)/\1/" /etc/locale.gen
locale-gen
echo "LANG=${LOCALE}" > /etc/locale.conf
echo "KEYMAP=${KEYMAP}" > /etc/vconsole.conf
echo "FONT=latarcyrheb-sun32" >> /etc/vconsole.conf
echo "$HOSTNAME" > /etc/hostname

cat > /etc/hosts <<EOF
127.0.0.1   localhost
::1         localhost
127.0.1.1   $HOSTNAME.localdomain $HOSTNAME
EOF

# --- System Update & Mirrors ---
log "Updating system and installing reflector..."
pacman -Syu --noconfirm --needed
pacman -S --noconfirm --needed reflector
reflector --country 'Brazil' --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist || \
reflector --latest 5 --protocol https --sort rate --save /etc/pacman.d/mirrorlist

# --- Install Desktop Environments (minimal) ---
log "Installing minimal desktop environments..."
pacman -S --noconfirm --needed \
    xorg $DM \
    gnome-shell gnome-control-center gnome-terminal nautilus \
    plasma-desktop dolphin konsole \
    cinnamon nemo-fileroller \
    lxqt breeze-icons \
    docker docker-buildx docker-compose \
    git nano code wget curl sudo zsh \
    gcc gdb ttf-sourcecodepro-nerd

# --- Remove software stores if present ---
pacman -Rns --noconfirm gnome-software discover mintinstall || true

# --- GPU Drivers ---
log "Installing GPU drivers for $GPU_VENDOR..."
case "$GPU_VENDOR" in
    nvidia)
        pacman -S --noconfirm --needed nvidia nvidia-utils nvidia-settings nvidia-container-toolkit cuda cuda-tools cudnn
        echo "options nvidia-drm modeset=1" > /etc/modprobe.d/nvidia.conf
        mkdir -p /etc/X11/xorg.conf.d
        echo -e 'Section "Device"\n  Identifier "Nvidia Card"\n  Driver "nvidia"\nEndSection' > /etc/X11/xorg.conf.d/10-nvidia.conf
        if [ "$DM" = "gdm" ]; then
            sed -i '/^#*WaylandEnable=/c\WaylandEnable=true' /etc/gdm/custom.conf || echo -e "[daemon]\nWaylandEnable=true" >> /etc/gdm/custom.conf
        fi
        nvidia-ctk runtime configure --runtime=docker
        ;;
    amd)
        pacman -S --noconfirm --needed xf86-video-amdgpu vulkan-radeon lib32-vulkan-radeon
        ;;
    intel)
        pacman -S --noconfirm --needed mesa vulkan-intel lib32-vulkan-intel
        ;;
    *)
        log "Unknown GPU vendor: $GPU_VENDOR. Skipping driver install."
        ;;
esac

# --- Docker config ---
cat > /etc/docker/daemon.json <<EOF
{
  "runtimes": {
    "nvidia": {
      "args": [],
      "path": "nvidia-container-runtime"
    }
  },
  "dns": ["8.8.8.8", "8.8.4.4"]
}
EOF

# --- CPU Microcode ---
CPU_VENDOR=$(grep -m 1 "vendor_id" /proc/cpuinfo | awk '{print $3}')
if [ "$CPU_VENDOR" = "GenuineIntel" ]; then
    pacman -S --noconfirm --needed intel-ucode
elif [ "$CPU_VENDOR" = "AuthenticAMD" ]; then
    pacman -S --noconfirm --needed amd-ucode
fi

# --- Enable Services ---
systemctl enable NetworkManager $DM docker

# --- User Creation ---
log "Creating user '$USERNAME'..."
useradd -m -G wheel,docker -s /bin/zsh "$USERNAME"
echo "$USERNAME:$PASSWORD" | chpasswd
sed -i 's/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers

# --- Oh-My-Zsh ---
log "Installing Oh-My-Zsh for $USERNAME..."
runuser -l "$USERNAME" -c 'sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended'

# --- GRUB ---
log "Installing GRUB bootloader..."
pacman -S --noconfirm --needed grub efibootmgr
grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
grub-mkconfig -o /boot/grub/grub.cfg

log "Setup complete. You can now reboot!"
