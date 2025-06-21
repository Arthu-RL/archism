#!/bin/bash
set -e

### --- CONFIGURATION --- ###
DISK="/dev/sda"
HOSTNAME="archism"
USERNAME="arthur"
LOCALE="pt_BR.UTF-8"
TIMEZONE="America/Sao_Paulo"
UI="cinnamon"
KEYMAP="br-abnt2"

echo ">>> Wiping disk and creating partitions..."

# Partition the disk
sgdisk -Z $DISK
sgdisk -n 1:0:+512M -t 1:ef00 -c 1:EFI $DISK
sgdisk -n 2:0:0 -t 2:8300 -c 2:ROOT $DISK

# Format partitions
mkfs.fat -F32 ${DISK}1
mkfs.ext4 ${DISK}2

# Mount partitions
mount ${DISK}2 /mnt
mkdir /mnt/boot
mount ${DISK}1 /mnt/boot

# Install base
echo ">>> Installing base system..."
pacstrap /mnt base linux linux-firmware nano git zsh wget curl sudo networkmanager

# Generate fstab
genfstab -U /mnt >> /mnt/etc/fstab

# Copy second script to chroot
curl -L "https://raw.githubusercontent.com/YOUR_GITHUB/archism/main/1-arch-setup.sh" -o /mnt/root/1-arch-setup.sh
chmod +x /mnt/root/1-arch-setup.sh

# Enter chroot and run setup
arch-chroot /mnt /root/1-arch-setup.sh $USERNAME $HOSTNAME $LOCALE $TIMEZONE $UI $KEYMAP

# Finish
echo ">>> All done! Unmounting and rebooting..."
umount -R /mnt
reboot
