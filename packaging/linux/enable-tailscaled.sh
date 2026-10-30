#!/bin/sh
# Run as root by the package manager after install: have the Tailscale daemon start at boot
# (and now), so ONYKS Chalcedon can connect to the VPN. Never fails the installation.
if command -v systemctl >/dev/null 2>&1 && systemctl cat tailscaled.service >/dev/null 2>&1; then
    systemctl enable --now tailscaled || true
fi
exit 0
