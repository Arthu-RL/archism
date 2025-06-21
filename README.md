Perfect! Here's a polished `README.md` for your **Archism** project:

---

# 🧱 Archism

**One-shot Arch Linux installation and setup** for an empty disk (SSD/HDD).
Archism automates the **entire installation process**, from disk partitioning to a ready-to-use graphical environment, shell, drivers, and tools — in just **one command**.

---

## 🚀 Features

* 🧹 Auto-wipes and partitions your disk (GPT + UEFI)
* ⚙️ Installs the full Arch base system
* 🖥️ Sets up your chosen Desktop Environment (e.g., Cinnamon, GNOME, KDE, etc.)
* 🧩 Installs NVIDIA drivers, Docker, and developer tools
* 🌍 Configures locale, timezone, keymap
* 👤 Creates your user account with Zsh and Oh-My-Zsh
* 🔒 Enables LightDM, Docker, and NetworkManager services
* 🔁 Installs and configures GRUB EFI bootloader

---

## 🛠️ Requirements

* A system with UEFI firmware (not legacy BIOS)
* Internet access (Wi-Fi or Ethernet)
* A blank disk (e.g., `/dev/sda`) that will be erased!

---

## 🔧 Run Scripts

> Boot into the Arch ISO, connect to the internet, then run:

```sh
curl -O https://raw.githubusercontent.com/arthur/archism/main/0-auto-install.sh
chmod +x 0-auto-install.sh
sudo ./0-auto-install.sh
```

The script will:

1. Partition and format your disk
2. Install the base system
3. Chroot and run the post-install script
4. Set up everything automatically
5. Reboot into your new Arch desktop

---

## 📁 Files

| File                | Description                                                                |
| ------------------- | -------------------------------------------------------------------------- |
| `0-auto-install.sh` | Runs in the Arch ISO. Partitions, installs base system, chroots into Arch. |
| `1-arch-setup.sh`   | Runs *inside chroot*. Installs drivers, DE, tools, configures system.      |

---

## 🧪 Customization

You can edit the top variables of `0-auto-install.sh`:

```bash
DISK="/dev/sda"
HOSTNAME="archbox"
USERNAME="username"
LOCALE="en_US.UTF-8" # "pt_BR.UTF-8"
TIMEZONE="America/Sao_Paulo"
UI="cinnamon"   # Options: cinnamon, gnome, plasma, xfce4, etc.
KEYMAP="br-abnt2"
```

Add your own packages and services, as needed!

---

## ⚠️ Warning

> This script will **completely erase** the target disk.
> Make sure you **know what you're doing** and back up your data!

---

## 📷 Screenshots (Optional)

> **TODO screenshots of evidence**

---

## ❤️ License

MIT — Use freely, share, and modify.

---

Let me know if you want help:

* Creating this GitHub repo
* Making a **custom Arch ISO image** that includes your scripts by default
* Adding Wi-Fi detection, LUKS encryption, or Btrfs snapshots

I can also make an interactive menu version (`archism-tui`) for advanced control.
