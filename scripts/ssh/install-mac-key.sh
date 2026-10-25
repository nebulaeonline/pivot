#!/usr/bin/env bash
# Append the host validation public key for the current Mac user; no sudo/settings changes.
set -eu
python3 /Volumes/RobMac/coding/pivot/scripts/ssh/install_validation_key.py
