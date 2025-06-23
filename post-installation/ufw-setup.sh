#!/bin/bash
set -euo pipefail

pacman -Sycv --noconfirm iptables-nft ufw

systemctl enable ufw

ufw enable