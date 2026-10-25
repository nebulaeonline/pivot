#!/usr/bin/env bash
# Frozen read-only collector; run inside the Ubuntu desktop terminal, without sudo.
set -eu
cd /mnt/j/coding/pivot/artifacts/ubuntu-gui-preparation/session-collector-20261025
printf '%s  %s\n' 1297f43d656e08b9d3b1220c12915adccd9f8d209c611f3e387c9067248ff600 ubuntu_gui_preparation.py | sha256sum -c -
python3 ubuntu_gui_preparation.py
