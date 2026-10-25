"""Append the validation public key; preserve existing authorized keys and access."""
import base64
import hashlib
import os
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from inspect_endpoint import main as inspect_endpoint

EXPECTED='m7bYetRRoU8I+0PkXxg9WPI4xrz2808/eAx+tvkonhI'
def main():
 if sys.platform not in ['linux','darwin']:raise SystemExit('Run as the intended normal Linux/macOS user, without sudo.')
 if os.getuid()==0:raise SystemExit('Use your normal guest user, not root/sudo.')
 root=Path(__file__).resolve().parents[2]
 text=(root/'artifacts/validation-ssh/validation.pub').read_text().strip()
 parts=text.split()
 if len(parts)<2 or parts[0]!='ssh-ed25519' or '\n' in text:raise ValueError('Expected one Ed25519 public key')
 blob=base64.b64decode(parts[1],validate=True)
 fingerprint=base64.b64encode(hashlib.sha256(blob).digest()).decode().rstrip('=')
 if fingerprint!=EXPECTED:raise ValueError('Unexpected validation public key; nothing changed')
 folder=Path.home()/'.ssh';authorized=folder/'authorized_keys'
 if folder.is_symlink() or authorized.is_symlink():raise ValueError('Refusing symlinked SSH key destination')
 folder.mkdir(mode=0o700,exist_ok=True);folder.chmod(0o700)
 previous=authorized.read_text() if authorized.exists() else ''
 present=any(len(p:=line.split())>=2 and p[0]==parts[0] and p[1]==parts[1] for line in previous.splitlines())
 if not present:
  with authorized.open('a',encoding='utf-8') as stream:
   if previous and not previous.endswith('\n'):stream.write('\n')
   stream.write(text+'\n')
 authorized.chmod(0o600)
 print('Validation key already installed.' if present else 'Validation public key installed; existing keys preserved.')
 return inspect_endpoint()
if __name__=='__main__':raise SystemExit(main())
