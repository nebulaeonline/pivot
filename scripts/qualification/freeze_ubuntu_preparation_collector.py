"""Freeze metadata collector only; no guest collection or GUI execution."""
import hashlib
import json
from pathlib import Path

root=Path(__file__).resolve().parents[2]
base=root/'artifacts/ubuntu-gui-preparation'
packet=base/'session-collector-20261025';packet.mkdir(parents=True)
source=root/'scripts/qualification/ubuntu_gui_preparation.py'
payload=source.read_bytes();target=packet/source.name;target.write_bytes(payload)
digest=hashlib.sha256(payload).hexdigest()
receipt={'scope':'read-only Ubuntu session collection; no activation',
 'files':[{'path':source.name,'sha256':digest}]}
(packet/'collector-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
launcher=root/'scripts/qualification/ubuntu-gui-preparation.sh'
launcher.write_text('''#!/usr/bin/env bash
# Frozen read-only collector; run inside the Ubuntu desktop terminal, without sudo.
set -eu
cd /mnt/j/coding/pivot/artifacts/ubuntu-gui-preparation/session-collector-20261025
printf '%s  %s\\n' '''+digest+''' ubuntu_gui_preparation.py | sha256sum -c -
python3 ubuntu_gui_preparation.py
''',encoding='utf-8',newline='\n')
(base/'collector-packaging.json').write_text(json.dumps({'collector_sha256':digest,
 'launcher_sha256':hashlib.sha256(launcher.read_bytes()).hexdigest(),
 'pure_non_GUI_tests':3,'actual_guest_collection':'pending','GUI_executed':False,
 'build_or_publish_executed':False,'disabled_GUI_runner_packet':'pending actual session evidence'},indent=2)+'\n')
print(digest)
