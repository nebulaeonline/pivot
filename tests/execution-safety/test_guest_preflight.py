"""Collector parsing/classification only; no actual guest, toolchain or clipboard."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('guest_preflight',ROOT/'scripts/qualification/guest_preflight.py')
guest = importlib.util.module_from_spec(spec);spec.loader.exec_module(guest)


class GuestPreflightTests(unittest.TestCase):
    def test_windows_process_architecture_distinguishes_native_from_emulated(self):
        self.assertEqual(guest.windows_machine_architectures(0,0x8664),('x64','x64'))
        self.assertEqual(guest.windows_machine_architectures(0,0xaa64),('arm64','arm64'))
        self.assertEqual(guest.windows_machine_architectures(0x8664,0xaa64),('arm64','x64'))
        self.assertEqual(guest.windows_machine_architectures(0x14c,0x8664),('x64','x86'))
        self.assertEqual(guest.windows_machine_architectures(99,0xaa64),('arm64','Unknown'))

    def test_native_elf_and_pe_headers_and_bad_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'image'
            for machine,want in [(62,'x64'),(183,'arm64'),(99,'Unknown')]:
                head=bytearray(64);head[:4]=b'\x7fELF';head[5]=1;head[18:20]=machine.to_bytes(2,'little');p.write_bytes(head)
                self.assertEqual(guest.image_arch(p),want)
            for machine,want in [(0x8664,'x64'),(0xaa64,'arm64'),(0,'Unknown')]:
                head=bytearray(64);head[:2]=b'MZ';head[60:64]=(64).to_bytes(4,'little');p.write_bytes(head+b'PE\0\0'+machine.to_bytes(2,'little'))
                self.assertEqual(guest.image_arch(p),want)
            for data in [b'',b'MZ',b'\x7fELF',b'not an image']:
                p.write_bytes(data);self.assertEqual(guest.image_arch(p),'Unknown')
            self.assertEqual(guest.image_arch(Path(tmp)/'missing'),'Unknown')

    def test_no_probe_no_native_root_or_wrong_arch_never_qualify(self):
        checks={key:{'status':'Verified'} for key in ['rust','cargo','dotnet','native-linker']}
        self.assertEqual(guest.capability(checks,'x64','x64',True,{}),'Blocked')
        probes={key:{'status':'Verified'} for key in ['rust-build','rust-execute','dotnet-build','dotnet-execute']}
        self.assertEqual(guest.capability(checks,'x64','x64',False,probes),'Blocked')
        self.assertEqual(guest.capability(checks,'arm64','x64',True,probes),'Blocked')
        self.assertEqual(guest.capability(checks,'Unknown','Unknown',True,probes),'Blocked')
        self.assertEqual(guest.capability(checks,'x64','x64',True,{'rust-build':{'status':'Verified'}}),'Blocked')
        self.assertEqual(guest.capability(checks,'x64','x64',True,probes),'Verified')

    def test_missing_tools_and_failed_probe_are_distinct(self):
        checks={key:{'status':'Verified'} for key in ['rust','cargo','dotnet','native-linker']}
        probes={key:{'status':'Verified'} for key in ['rust-build','rust-execute','dotnet-build','dotnet-execute']}
        checks['cargo']['status']='NotConfigured'
        self.assertEqual(guest.capability(checks,'x64','x64',True,probes),'NotConfigured')
        checks['cargo']['status']='Blocked'
        self.assertEqual(guest.capability(checks,'x64','x64',True,probes),'Blocked')
        checks['cargo']['status']='Verified';probes['dotnet-execute']['status']='Failed'
        self.assertEqual(guest.capability(checks,'x64','x64',True,probes),'Failed')

    def test_missing_command_and_timeout_do_not_turn_into_passes(self):
        with patch('subprocess.run',side_effect=FileNotFoundError('missing')):
            self.assertEqual(guest.run(['synthetic'])['status'],'NotConfigured')
        with patch('subprocess.run',side_effect=guest.subprocess.TimeoutExpired('synthetic',45)):
            self.assertEqual(guest.run(['synthetic'])['status'],'Blocked')

    @unittest.skipIf(guest.os.name=='nt','Linux findmnt branch')
    def test_network_and_unknown_filesystems_are_blocked(self):
        for kind,source in [('cifs','//host/share'),('9p','drvfs'),('overlay','overlay'),('ext4','/not-a-device')]:
            with patch.object(guest,'run',return_value={'status':'Verified','stdout':guest.json.dumps({'filesystems':[{'fstype':kind,'source':source,'target':'/fixture'}]})}):
                self.assertFalse(guest.filesystem(Path('/fixture'))['native_work_root'])


if __name__=='__main__':unittest.main()
