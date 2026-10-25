#!/usr/bin/env bash
# Frozen tests and read-only live adapters only; no GUI or builds. No sudo.
set -eu
python3 /mnt/j/coding/pivot/artifacts/ubuntu-managed-implementation/check_guest_non_gui.py
