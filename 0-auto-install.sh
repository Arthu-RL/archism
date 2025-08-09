#!/bin/bash
set -euo pipefail

# --- Logging helper ---
log() { echo -e "\033[1;32m[$(date +'%H:%M:%S')] $1\033[0m"; }
err() { echo -e "\033[1;31m[ERROR $(date +'%H:%M:%S')] $1\033[0m" >&2; }

log "Welcome to Archism auto-installer"
echo

# --- Prompt helper ---
prompt_default() {
    local varname=$1
    local prompt=$2
    local default=$3
    read -rp "$prompt [$default]: " input
    export "$varname"="${input:-$default}"
}

# --- Collect settings ---
prompt_default DISK "Target disk (will be ERASED)" "/dev/sda"
if [ ! -b "$DISK" ]; then
    err "'$DISK' is not a valid block device."
    exit 1
fi

prompt_default HOSTNAME "Hostname" "archism"

while true; do
    read -rp "Username (required): " USERNAME
    [[ -n "$USERNAME" ]] && break
    err "Username cannot be empty."
done

prompt_default LOCALE "Locale" "en_US.UTF-8"
prompt_default TIMEZONE "Timezone (Region/City)" "America/Sao_Paulo"
prompt_default KEYMAP "Keyboard layout (KEYMAP)" "br-abnt2"
prompt_default DM "Display Manager argument (e.g., gdm, sddm, lightdm)" "gdm"

# GPU detection
GPU_VENDOR="unknown"
if lspci | grep -qi nvidia; then
    GPU_VENDOR="nvidia"
elif lspci | grep -qi amd; then
    GPU_VENDOR="amd"
elif lspci | grep -qi intel; then
    GPU_VENDOR="intel"
fi

# --- Summary ---
echo
log "Summary:"
echo "Disk:         $DISK"
echo "Hostname:     $HOSTNAME"
echo "Username:     $USERNAME"
echo "Locale:       $LOCALE"
echo "Timezone:     $TIMEZONE"
echo "Keymap:       $KEYMAP"
echo "DM:           $DM"
echo "GPU Vendor:   $GPU_VENDOR"
echo

read -p "Continue with these settings? (y/n): " CONFIRM
[[ "$CONFIRM" =~ ^[Yy]$ ]] || exit 1

# --- Password ---
while true; do
    read -sp "Enter password for user '$USERNAME': " password1; echo
    read -sp "Confirm password: " password2; echo
    if [ "$password1" == "$password2" ]; then
        PASSWORD="$password1"
        break
    else
        err "Passwords do not match, please try again."
    fi
done

# --- Clock sync ---
log "Synchronizing system clock..."
timedatectl set-ntp true

# --- Internet check ---
if ! ping -c 1 archlinux.org &>/dev/null; then
    err "No internet connection. Please connect and try again."
    exit 1
fi

# --- Full install? ---
read -p "Run full install (wipe disk and install base system)? [y/N]: " DO_FULL
if [[ "$DO_FULL" =~ ^[Yy]$ ]]; then
    log "Wiping disk and creating GPT partitions..."
    sgdisk -Z "$DISK" || { err "sgdisk failed."; exit 1; }
    sgdisk -n 1:0:+2048M -t 1:ef00 -c 1:EFI "$DISK"
    sgdisk -n 2:0:0    -t 2:8300 -c 2:ROOT "$DISK"

    PART_BOOT="$(ls ${DISK}* | grep -E "${DISK}(p)?1$")"
    PART_ROOT="$(ls ${DISK}* | grep -E "${DISK}(p)?2$")"

    if [ ! -d /sys/firmware/efi ]; then
        err "System is not in UEFI mode!"
        exit 1
    fi

    log "Formatting partitions..."
    mkfs.fat -F32 "$PART_BOOT"
    mkfs.ext4 -F "$PART_ROOT"

    log "Mounting filesystems..."
    mount "$PART_ROOT" /mnt
    mkdir -p /mnt/boot
    mount "$PART_BOOT" /mnt/boot

    log "Installing base system..."
    pacstrap /mnt --needed base linux linux-firmware nano git zsh wget curl sudo networkmanager
    log "Generating fstab..."
    genfstab -U /mnt >> /mnt/etc/fstab
else
    read -rp "Enter EFI partition (e.g., /dev/sda1): " PART_BOOT
    read -rp "Enter ROOT partition (e.g., /dev/sda2): " PART_ROOT
    log "Mounting existing partitions..."
    mount "$PART_ROOT" /mnt
    mkdir -p /mnt/boot
    mount "$PART_BOOT" /mnt/boot
fi

# --- Swap size based on RAM ---
TOTAL_RAM=$(grep MemTotal /proc/meminfo | awk '{print int($2/1024)}')
SWAP_SIZE=$((TOTAL_RAM < 8000 ? TOTAL_RAM : 8000))
log "Creating $SWAP_SIZE MB swapfile..."
fallocate -l "${SWAP_SIZE}M" /mnt/swapfile
chmod 600 /mnt/swapfile
mkswap /mnt/swapfile
echo '/swapfile none swap defaults 0 0' >> /mnt/etc/fstab

# --- Stage 2 ---
log "Downloading second stage setup script..."
if ! curl -fL "https://raw.githubusercontent.com/Arthu-RL/archism/main/1-arch-setup.sh" -o /mnt/root/1-arch-setup.sh; then
    err "Failed to download stage 2 setup script."
    exit 1
fi
chmod +x /mnt/root/1-arch-setup.sh

log "Entering chroot and launching setup..."
arch-chroot /mnt /root/1-arch-setup.sh "$USERNAME" "$PASSWORD" "$HOSTNAME" "$LOCALE" "$TIMEZONE" "$DM" "$KEYMAP" "$GPU_VENDOR"

log "Unmounting and rebooting in 5 seconds..."
umount -R /mnt
sleep 5
reboot
