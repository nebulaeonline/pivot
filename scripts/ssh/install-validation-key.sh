#!/usr/bin/env bash
# Installs only the host validation public key for the current guest user.
# No sudo, service changes, firewall edits or removal of existing keys.
set -eu
python3 /mnt/j/coding/pivot/scripts/ssh/install_validation_key.py
