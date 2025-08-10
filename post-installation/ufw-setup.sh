#!/bin/bash

pacman -Syv --noconfirm iptables-nft ufw

systemctl enable ufw

ufw enable