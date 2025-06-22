#!/bin/bash
set -euo pipefail

echo ">>> Welcome to Archism auto-installer"
echo

### --- HELPER FUNCTION --- ###
prompt_default() {
    local varname=$1
    local prompt=$2
    local default=$3
    read -rp "$prompt [$default]: " input
    export "$varname"="${input:-$default}"
}

### --- CONFIGURATION INPUT --- ###

prompt_default DISK "Target disk (will be ERASED)" "/dev/sda"
prompt_default HOSTNAME "Hostname" "archism"

while true; do
    read -rp "Username (required): " USERNAME
    [[ -n "$USERNAME" ]] && break
    echo "Username cannot be empty."
done

prompt_default LOCALE "Locale" "en_US.UTF-8"
prompt_default TIMEZONE "Timezone (Region/City)" "America/Sao_Paulo"
prompt_default KEYMAP "Keyboard layout (KEYMAP)" "br-abnt2"
prompt_default UI "Desktop Environment [gnome, cinnamon, plasma, xfce4, etc.]" "gnome"

### --- CONTINUE WITH INSTALLATION --- ###
echo
echo ">>> Summary:"
echo "Disk:         $DISK"
echo "Hostname:     $HOSTNAME"
echo "Username:     $USERNAME"
echo "Locale:       $LOCALE"
echo "Timezone:     $TIMEZONE"
echo "Keymap:       $KEYMAP"
echo "UI:           $UI"
echo

read -p "Continue with these settings? (y/n): " CONFIRM
[[ "$CONFIRM" == "y" || "$CONFIRM" == "Y" ]] || exit 1

### --- SENSITIVE DATA --- ###
read -sp "Enter password for user '$USERNAME': " PASSWORD
echo

### --- CLOCK SYNC --- ###
echo ">>> Synchronizing system clock..."
timedatectl set-ntp true

### --- ASK TO RUN FULL INSTALL --- ###
read -p "Run full install (wipe disk and install base system)? [y/N]: " DO_FULL
if [[ "$DO_FULL" =~ ^[Yy]$ ]]; then
    ### --- PARTITIONING DISK --- ###
    echo ">>> Wiping disk and creating GPT partitions on $DISK..."
    sgdisk -Z "$DISK"
    sgdisk -n 1:0:+512M -t 1:ef00 -c 1:EFI "$DISK"
    sgdisk -n 2:0:0    -t 2:8300 -c 2:ROOT "$DISK"

    echo ">>> Formatting partitions..."
    mkfs.fat -F32 "${DISK}1"
    mkfs.ext4      "${DISK}2"

    echo ">>> Mounting file systems..."
    mount "${DISK}2" /mnt
    mkdir -p /mnt/boot
    mount "${DISK}1" /mnt/boot

    echo ">>> Installing base system..."
    pacstrap /mnt base linux linux-firmware nano git zsh wget curl sudo networkmanager

    echo ">>> Generating fstab..."
    genfstab -U /mnt >> /mnt/etc/fstab
else
    echo ">>> Skipping base install. Checking if /mnt is mounted..."

    if ! mountpoint -q /mnt; then
        echo ">>> Mounting existing partitions..."
        mount "${DISK}2" /mnt
        mkdir -p /mnt/boot
        mount "${DISK}1" /mnt/boot
    else
        echo ">>> /mnt is already mounted. Continuing..."
    fi

    echo ">>> You can chroot and rerun the setup script manually if needed."
fi

### --- SETUP SCRIPT --- ###
echo ">>> Downloading second stage setup script..."
curl -L "https://raw.githubusercontent.com/Arthu-RL/archism/main/1-arch-setup.sh" -o /mnt/root/1-arch-setup.sh
chmod +x /mnt/root/1-arch-setup.sh

echo ">>> Entering chroot and launching setup..."
arch-chroot /mnt /root/1-arch-setup.sh "$USERNAME" "$PASSWORD" "$HOSTNAME" "$LOCALE" "$TIMEZONE" "$UI" "$KEYMAP"

echo ">>> Unmounting and rebooting in 5 seconds..."
umount -R /mnt
sleep 5
reboot
