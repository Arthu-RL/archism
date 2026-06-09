# 📦 `ARCH_BOOT_USB.md`

A step-by-step guide to **download**, **verify**, **flash** the official Arch Linux ISO, and **run your custom installer** directly from a remote source.

---

## 📅 Set ISO Version (Current Release)

```sh
export ARCH_INSTALL_DATE=2026.06.01
```

---

## 🌐 Download ISO + Checksum File

```sh
wget https://mirror.rackspace.com/archlinux/iso/$ARCH_INSTALL_DATE/archlinux-$ARCH_INSTALL_DATE-x86_64.iso
wget https://mirror.rackspace.com/archlinux/iso/$ARCH_INSTALL_DATE/sha256sums.txt
```

---

## 🔐 Verify the ISO

Ensure the downloaded ISO is valid and not corrupted:

```sh
sha256sum -c sha256sums.txt 2>&1 | grep "archlinux-$ARCH_INSTALL_DATE-x86_64.iso"
```

✅ Output should show: `OK`

---

## 💾 Flash ISO to USB with `dd`

Before flash, make sure your device is umounted, if not, the flash will not work:

```sh
sudo umount /dev/sdX*
```

Now you can flash the iso in it:

```sh
sudo dd bs=4M if=archlinux-$ARCH_INSTALL_DATE-x86_64.iso of=/dev/sdX status=progress oflag=sync
```

> Replace `/dev/sdX` with the actual device (like `/dev/sda`, `/dev/sdb`) — **not** a partition like `/dev/sda1`.

---

## 🧠 `dd` Reference (Quick Summary)

| Option            | Description                                          |
| ----------------- | ---------------------------------------------------- |
| `if=FILE`         | Input file (e.g., ISO file)                          |
| `of=FILE`         | Output file (e.g., USB device like `/dev/sda`)       |
| `bs=4M`           | Block size (read/write 4MB chunks at a time)         |
| `status=progress` | Shows write progress in real time                    |
| `oflag=sync`      | Forces sync after each block (more reliable writing) |

---

## ⚠️ Warnings

* Double-check the device (`/dev/sdX`) before running `dd`.
* Flashing the wrong device can **erase your entire system**.

Use `lsblk` to safely identify your USB:

```sh
lsblk
```

---

## ✅ After Flashing

1. Wait until `dd` completes and returns to prompt.
2. Run `sync` to flush write cache:

```sh
sync
```

3. Remove USB safely:

```sh
udisksctl power-off -b /dev/sdX
```

---

## 🚀 Step 6: Boot and Run the Installer Live

After boot into flashdrive, you will see the arch linux command line. Follow instructions below for complete installation.

### 1. Connect to Internet

If you are using a network cable (Ethernet), the connection will be automatically established. In case of WIFI, use `iwctl` utility:

```sh
iwctl

# Inside iwctl, locate your interfae, scan and connect
# station wlan0 scan
# station wlan0 get-networks
# station wlan0 connect NOME_DA_SUA_REDE
# quit
```

Network test command:

```sh
ping -c 3 archlinux.org
```

### 2. Download the binary

Download the binary and execute it to configure a arch linux installation:

```sh
curl -LO https://github.com/Arthu-RL/archism/releases/download/1.0.0/archism-runner

chmod +x archism-installer

./archism-installer
```
