#!/bin/bash
set -euo pipefail

USERNAME="$1"
PASSWORD="$2"
HOSTNAME="$3"
LOCALE="$4"
TIMEZONE="$5"
UI="$6"
KEYMAP="$7"

echo ">>> Setting timezone, locale, hostname..."
ln -sf /usr/share/zoneinfo/$TIMEZONE /etc/localtime
hwclock --systohc
sed -i "s/^#\(${LOCALE}\)/\1/" /etc/locale.gen
locale-gen
echo "LANG=${LOCALE}" > /etc/locale.conf

echo "KEYMAP=${KEYMAP}" > /etc/vconsole.conf
echo "FONT=latarcyrheb-sun32" >> /etc/vconsole.conf

echo "$HOSTNAME" > /etc/hostname

echo ">>> Configuring /etc/hosts..."
cat > /etc/hosts <<EOF
127.0.0.1   localhost
::1         localhost
127.0.1.1   $HOSTNAME.localdomain $HOSTNAME
EOF

echo ">>> Installing system packages..."
pacman -Syu --noconfirm

# Determine Display Manager
case $UI in
    gnome) DM="gdm";;
    plasma) DM="sddm";;
    *) DM="lightdm";;
esac

# Internet optimization tool - keep your mirror list fresh and optimized for your location and internet speed
pacman -S --noconfirm reflector
reflector --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist

# Install GNOME, and supporting tools for dev
pacman -S --noconfirm --needed $UI gnome-control-center \
    xorg $DM docker docker-buildx docker-compose git nano code wget curl sudo zsh \
    gcc gdb ttf-sourcecodepro-nerd iptables-nft ufw

# Install NVIDIA, and supporting tools for docker
pacman -S --noconfirm --needed nvidia nvidia-utils nvidia-settings nvidia-container-toolkit cuda cuda-tools cudnn

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

# NVIDIA + GNOME Wayland Compatibility (uses default Wayland)
echo ">>> Configuring NVIDIA for GNOME on Wayland..."
mkdir -p /etc/modprobe.d
echo "options nvidia-drm modeset=1" > /etc/modprobe.d/nvidia.conf

mkdir -p /etc/X11/xorg.conf.d
echo -e 'Section "Device"\n  Identifier "Nvidia Card"\n  Driver "nvidia"\nEndSection' > /etc/X11/xorg.conf.d/10-nvidia.conf

# Ensure Wayland is enabled
grep -q '^WaylandEnable=' /etc/gdm/custom.conf \
  && sed -i 's/^WaylandEnable=.*/WaylandEnable=true/' /etc/gdm/custom.conf \
  || echo -e "[daemon]\nWaylandEnable=true" >> /etc/gdm/custom.conf

# CPU microcode
CPU_VENDOR=$(grep -m 1 "vendor_id" /proc/cpuinfo | awk '{print $3}')
if [ "$CPU_VENDOR" = "GenuineIntel" ]; then
    pacman -S --noconfirm intel-ucode
elif [ "$CPU_VENDOR" = "AuthenticAMD" ]; then
    pacman -S --noconfirm amd-ucode
fi

# Enable services
systemctl enable NetworkManager
systemctl enable $DM
systemctl enable docker
systemctl enable ufw

# Ensure nf_conntrack is loaded (needed for iptables and ufw)
echo nf_conntrack > /etc/modules-load.d/conntrack.conf

# Activate firewall
ufw enable

# User creation
echo ">>> Creating user '$USERNAME'..."
useradd -m -G wheel,docker -s /bin/zsh "$USERNAME"
echo "$USERNAME:$PASSWORD" | chpasswd

# Sudoers config
sed -i 's/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers

# Oh-My-Zsh install
echo ">>> Installing Oh-My-Zsh for $USERNAME..."
runuser -l "$USERNAME" -c 'sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended'

# GRUB install
echo ">>> Installing GRUB bootloader..."
pacman -S --noconfirm grub efibootmgr
grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
grub-mkconfig -o /boot/grub/grub.cfg

echo ">>> Setup complete. You can now reboot!"
exit
