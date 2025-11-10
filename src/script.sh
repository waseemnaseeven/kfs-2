#!/bin/bash

# general 
sudo apt-get update -y
sudo apt-get upgrade -y
sudo apt-get install -y git gnupg vim make wget curl openssh-server ufw net-tools
sudo apt-get install -y build-essential
sudo apt-get install software-properties-common -y
sudo apt-get install xorriso -y
sudo apt-get install qemu-system -y

# firewall
sudo ufw allow OpenSSH
sudo ufw enable

# oh my zsh
sudo apt-get install -y zsh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

# rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# users && groups
sudo usermod -aG sudo $USER
sudo usermod -aG docker $USER
sudo usermod -aG $USER $USER
sudo hostnamectl set-hostname $USER

echo "under the project folder, do the following: "
cargo rustc -- -C link-arg=-nostartfiles


