#!/bin/sh
# Run as root by the package manager after install. Never fails the installation.
#  - the Tailscale daemon starts at boot (and now), so ONYKS Chalcedon can connect to the VPN;
#  - the PostgreSQL ODBC driver (needed by the Altium DbLib / KiCad kicad_dbl) must be registered.
if command -v systemctl >/dev/null 2>&1 && systemctl cat tailscaled.service >/dev/null 2>&1; then
    systemctl enable --now tailscaled || true
else
    echo "ONYKS Chalcedon: tailscaled.service not found - install Tailscale (https://tailscale.com/download)." >&2
fi

if ! command -v odbcinst >/dev/null 2>&1 || ! odbcinst -q -d 2>/dev/null | grep -qi postgresql; then
    echo "ONYKS Chalcedon: the PostgreSQL ODBC driver is not registered (odbc-postgresql / postgresql-odbc / psqlodbc)." >&2
fi
exit 0
