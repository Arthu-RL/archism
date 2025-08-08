#!/bin/bash
set -euo pipefail

# --- Logging ---
log() { echo -e "\033[1;32m[$(date +'%H:%M:%S')] $1\033[0m"; }

# --- Validate arguments ---
if [ "$#" -ne 7 ]; then
    echo "Usage: $0 <username> <password> <hostname> <locale> <timezone> <display_manager> <keymap>"
    exit 1
fi

USERNAME="$1"
PASSWORD="$2"
HOSTNAME="$3"
LOCALE="$4"
TIMEZONE="$5"
DM="$6"
KEYMAP="$7"

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

# --- Hosts ---
cat > /etc/hosts <<EOF
127.0.0.1   localhost
::1         localhost
127.0.1.1   $HOSTNAME.localdomain $HOSTNAME
EOF

# --- System Update & Mirrorlist ---
log "Updating system and installing reflector..."
pacman -Syu --noconfirm --needed
pacman -S --noconfirm --needed reflector
reflector --country 'Brazil' --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist || \
reflector --latest 5 --protocol https --sort rate --save /etc/pacman.d/mirrorlist

# --- Install Desktop & Tools ---
log "Installing desktop environments, dev tools, Docker, NVIDIA..."
pacman -S --noconfirm --needed \
    xorg $DM \
    gnome gnome-extra gnome-control-center \
    plasma kde-applications \
    cinnamon nemo-fileroller \
    lxqt breeze-icons \
    docker docker-buildx docker-compose \
    git nano code wget curl sudo zsh \
    gcc gdb ttf-sourcecodepro-nerd \
    nvidia nvidia-utils nvidia-settings \
    nvidia-container-toolkit cuda cuda-tools cudnn

# --- NVIDIA Config ---
if lspci | grep -i nvidia &>/dev/null; then
    log "Configuring NVIDIA..."
    echo "options nvidia-drm modeset=1" > /etc/modprobe.d/nvidia.conf
    mkdir -p /etc/X11/xorg.conf.d
    echo -e 'Section "Device"\n  Identifier "Nvidia Card"\n  Driver "nvidia"\nEndSection' > /etc/X11/xorg.conf.d/10-nvidia.conf
    if [ "$DM" = "gdm" ]; then
        sed -i '/^#*WaylandEnable=/c\WaylandEnable=true' /etc/gdm/custom.conf || echo -e "[daemon]\nWaylandEnable=true" >> /etc/gdm/custom.conf
    fi
fi

# --- Docker Config ---
nvidia-ctk runtime configure --runtime=docker
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
