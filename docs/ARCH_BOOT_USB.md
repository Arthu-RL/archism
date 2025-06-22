# 📦 `ARCH_BOOT_USB.md`

A step-by-step guide to **download**, **verify**, and **flash** the latest Arch Linux ISO to a USB drive.

---

## 📅 Set ISO Version (Current Release)

```sh
export ARCH_INSTALL_DATE=2025.06.01
```

---

## 🌐 Download ISO + Checksum File

```sh
wget https://mirror.rackspace.com/archlinux/iso/${ARCH_INSTALL_DATE}/archlinux-${ARCH_INSTALL_DATE}-x86_64.iso
wget https://mirror.rackspace.com/archlinux/iso/${ARCH_INSTALL_DATE}/sha256sums.txt
```

---

## 🔐 Verify the ISO

Ensure the downloaded ISO is valid and not corrupted:

```sh
sha256sum -c sha256sums.txt 2>&1 | grep "archlinux-${ARCH_INSTALL_DATE}-x86_64.iso"
```

✅ Output should show: `OK`

---

## 💾 Flash ISO to USB with `dd`

```sh
sudo dd bs=4M if=archlinux-${ARCH_INSTALL_DATE}-x86_64.iso of=/dev/sdX status=progress oflag=sync
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

1. Wait until `dd` completes and returns to prompt
2. Run `sync` to flush write cache:

   ```sh
   sync
   ```
3. Remove USB safely:

   ```sh
   udisksctl power-off -b /dev/sdX
   ```