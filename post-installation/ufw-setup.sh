#!/bin/bash
set -euo pipefail

pacman -Syv --noconfirm iptables-nft ufw

systemctl enable ufw

ufw enable