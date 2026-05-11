---
title: "Nova Docs"
description: ""
---


## Installation

### Homebrew

::: tabs
== tab "macOS"
```sh
brew install --cask pmqueiroz/tap/nova
```

== tab "Linux"
```sh
brew install pmqueiroz/tap/nova
```
:::

### Winget (Windows)

```sh
winget install pmqueiroz.Nova
```

### Scoop (Windows)

```sh
scoop bucket add pmqueiroz https://github.com/pmqueiroz/scoop-bucket
scoop install nova
```

### AUR (Arch Linux)

```sh
yay -S nova-bin
```

### APT (Debian/Ubuntu)

```sh
curl -fsSL https://pmqueiroz.github.io/apt-repo/KEY.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/nova.gpg
echo "deb [trusted=yes arch=amd64] https://pmqueiroz.github.io/apt-repo stable main" \
  | sudo tee /etc/apt/sources.list.d/nova.list
sudo apt update && sudo apt install nova
```

### DNF/RPM (Fedora/RHEL)

```sh
sudo curl -fsSL https://pmqueiroz.github.io/rpm-repo/nova.repo \
  -o /etc/yum.repos.d/nova.repo
sudo dnf install nova
```
