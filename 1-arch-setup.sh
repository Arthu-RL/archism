#!/bin/bash
set -e

USERNAME=$1
HOSTNAME=$2
LOCALE=$3
TIMEZONE=$4
UI=$5
KEYMAP=$6

# Set timezone, locale, hostname
ln -sf /usr/share/zoneinfo/$TIMEZONE /etc/localtime
hwclock --systohc
sed -i "s/^#${LOCALE}/${LOCALE}/" /etc/locale.gen
locale-gen
echo "LANG=${LOCALE}" > /etc/locale.conf
echo "KEYMAP=${KEYMAP}" > /etc/vconsole.conf
echo "$HOSTNAME" > /etc/hostname
echo "127.0.1.1    $HOSTNAME.localdomain $HOSTNAME" >> /etc/hosts

# Install drivers, UI, etc.
pacman -Syu --noconfirm
pacman -S --noconfirm $UI xorg xorg-xinit lightdm lightdm-gtk-greeter \
    docker nvidia nvidia-utils nvidia-settings git nano zsh sudo

# Enable services
systemctl enable NetworkManager
systemctl enable lightdm
systemctl enable docker

# Create user
useradd -m -G wheel,docker -s /bin/zsh $USERNAME
echo "%wheel ALL=(ALL:ALL) ALL" >> /etc/sudoers
echo "Set password for $USERNAME:"
passwd $USERNAME

# Oh-My-Zsh
runuser -l $USERNAME -c "
    sh -c \"\$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" --unattended
"

# Bootloader
pacman -S --noconfirm grub efibootmgr
grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
grub-mkconfig -o /boot/grub/grub.cfg
