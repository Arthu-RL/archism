#!/bin/bash
set -euo pipefail

### --- CONFIGURATION --- ###
DISK="/dev/sda"
HOSTNAME="archism"
USERNAME="username"
LOCALE="en_US.UTF-8"
TIMEZONE="America/Sao_Paulo"
UI="gnome"
KEYMAP="br-abnt2"

### --- SENSITIVE DATA --- ###
read -sp "Enter password for user '$USERNAME': " PASSWORD
echo

### --- CLOCK SYNC --- ###
echo ">>> Synchronizing system clock..."
timedatectl set-ntp true

### --- PARTITIONING DISK --- ###
echo ">>> Wiping disk and creating GPT partitions on $DISK..."
sgdisk -Z $DISK
sgdisk -n 1:0:+512M -t 1:ef00 -c 1:EFI $DISK
sgdisk -n 2:0:0 -t 2:8300 -c 2:ROOT $DISK

echo ">>> Formatting partitions..."
mkfs.fat -F32 ${DISK}1
mkfs.ext4 ${DISK}2

echo ">>> Mounting file systems..."
mount ${DISK}2 /mnt
mkdir -p /mnt/boot
mount ${DISK}1 /mnt/boot

### --- BASE SYSTEM --- ###
echo ">>> Installing base system..."
pacstrap /mnt base linux linux-firmware nano git zsh wget curl sudo networkmanager

echo ">>> Generating fstab..."
genfstab -U /mnt >> /mnt/etc/fstab

echo ">>> Downloading second stage setup script..."
curl -L "https://raw.githubusercontent.com/arthur/archism/main/1-arch-setup.sh" -o /mnt/root/1-arch-setup.sh
chmod +x /mnt/root/1-arch-setup.sh

echo ">>> Entering chroot and launching setup..."
arch-chroot /mnt /root/1-arch-setup.sh "$USERNAME" "$PASSWORD" "$HOSTNAME" "$LOCALE" "$TIMEZONE" "$UI" "$KEYMAP"

echo ">>> Unmounting and rebooting in 5 seconds..."
umount -R /mnt
sleep 5
reboot
