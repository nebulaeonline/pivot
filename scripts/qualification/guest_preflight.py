"""Optional guest-local headless preflight. No remote execution, GUI or installation.

Metadata by default; --probe opts into tiny native Rust/.NET compile/execute checks
in a fresh local report directory. This never runs Pivot acceptance or grants a permit.
"""
import argparse
import ctypes
import datetime
import getpass
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import struct
import subprocess
import sys
import uuid


def image_arch(path):
    """Inspect native file header rather than inferring architecture from its name."""
    try:
        with Path(path).open('rb') as stream:
            head = stream.read(64)
            if head[:4] == b'\x7fELF' and len(head) >= 20:
                order = '<' if head[5] == 1 else '>' if head[5] == 2 else None
                if order is None: return 'Unknown'
                return {62:'x64', 183:'arm64', 3:'x86'}.get(struct.unpack(order+'H', head[18:20])[0], 'Unknown')
            if head[:2] == b'MZ' and len(head) == 64:
                offset = int.from_bytes(head[60:64], 'little')
                if offset > 16*1024*1024: return 'Unknown'
                stream.seek(offset); signature = stream.read(6)
                if signature[:4] == b'PE\0\0':
                    return {0x8664:'x64', 0xaa64:'arm64', 0x14c:'x86'}.get(int.from_bytes(signature[4:6], 'little'), 'Unknown')
    except OSError:
        pass
    return 'Unknown'


def windows_machine_architectures(process_machine, native_machine):
    mapping = {0x8664:'x64', 0xaa64:'arm64', 0x14c:'x86'}
    native = mapping.get(native_machine, 'Unknown')
    return native, native if process_machine == 0 else mapping.get(process_machine, 'Unknown')


def windows_architectures():
    # Query the executing process, including packaged Python aliases whose file
    # header may be unreadable. Never infer process architecture from pointer size.
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetCurrentProcess.restype = ctypes.c_void_p
    kernel.IsWow64Process2.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ushort), ctypes.POINTER(ctypes.c_ushort)]
    process, native = ctypes.c_ushort(), ctypes.c_ushort()
    if not kernel.IsWow64Process2(kernel.GetCurrentProcess(), ctypes.byref(process), ctypes.byref(native)):
        raise OSError(ctypes.get_last_error(), 'IsWow64Process2 failed')
    return windows_machine_architectures(process.value,native.value)


def kernel_arch():
    if os.name == 'nt':
        return windows_architectures()[0]
    return {'x86_64':'x64', 'aarch64':'arm64', 'arm64':'arm64'}.get(platform.machine().lower(), 'Unknown')


def run(command, cwd=None, env=None):
    try:
        result = subprocess.run(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=45)
        return {'command':command, 'status':'Verified' if result.returncode == 0 else 'Failed',
                'exit_code':result.returncode, 'stdout':result.stdout, 'stderr':result.stderr}
    except FileNotFoundError as error:
        return {'command':command, 'status':'NotConfigured', 'reason':str(error)}
    except (OSError, subprocess.TimeoutExpired) as error:
        return {'command':command, 'status':'Blocked', 'reason':str(error)}


def filesystem(path):
    if os.name == 'nt':
        from ctypes import wintypes
        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        kernel.GetVolumePathNameW.argtypes = [wintypes.LPCWSTR, wintypes.LPWSTR, wintypes.DWORD]
        kernel.GetDriveTypeW.argtypes = [wintypes.LPCWSTR]
        kernel.GetVolumeInformationW.argtypes = [wintypes.LPCWSTR, wintypes.LPWSTR, wintypes.DWORD,
            ctypes.POINTER(wintypes.DWORD), ctypes.POINTER(wintypes.DWORD), ctypes.POINTER(wintypes.DWORD), wintypes.LPWSTR, wintypes.DWORD]
        volume = ctypes.create_unicode_buffer(1024); kind = ctypes.create_unicode_buffer(128)
        if not kernel.GetVolumePathNameW(str(path), volume, len(volume)): raise OSError('Cannot resolve volume')
        drive_type = kernel.GetDriveTypeW(volume.value)
        if not kernel.GetVolumeInformationW(volume.value, None, 0, None, None, None, kind, len(kind)):
            raise OSError('Cannot read volume filesystem')
        return {'type':kind.value, 'volume':volume.value, 'drive_type':drive_type,
                'native_work_root':drive_type == 3 and kind.value.upper() == 'NTFS'}
    query = run(['findmnt','--json','--target',str(path),'--output','TARGET,SOURCE,FSTYPE'])
    if query['status'] != 'Verified': return {'native_work_root':False,'query':query}
    data = json.loads(query['stdout'])['filesystems'][0]
    # Conservative allowlist: mounted shares/DrvFS/overlay/unknown are not native proof.
    return {'type':data['fstype'], 'mount':data['target'], 'source':data['source'],
            'native_work_root':data['fstype'] in ['ext4','xfs','btrfs'] and data['source'].startswith('/dev/')}


def capability(checks, native_arch, python_arch, local_root, probes):
    """Conservative candidate state, never a final headless/GUI approval."""
    prerequisites = ['rust','cargo','dotnet','native-linker']
    if not local_root or native_arch == 'Unknown' or python_arch != native_arch:
        return 'Blocked'
    if any(checks[k]['status'] == 'NotConfigured' for k in prerequisites): return 'NotConfigured'
    if any(checks[k]['status'] != 'Verified' for k in prerequisites): return 'Blocked'
    if not probes: return 'Blocked'  # Explicit native executable proof is still absent.
    if any(p['status'] == 'Failed' for p in probes.values()): return 'Failed'
    if any(p['status'] != 'Verified' for p in probes.values()): return 'Blocked'
    if set(probes) != {'rust-build','rust-execute','dotnet-build','dotnet-execute'}: return 'Blocked'
    return 'Verified'  # Candidate evidence only; reviewer must bind intended guest identity.


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-root', type=Path, required=True, help='Existing guest-local NTFS/Linux disk directory')
    parser.add_argument('--probe', action='store_true', help='Compile and execute tiny private Rust/.NET probes; no packages installed')
    args = parser.parse_args()
    parent = args.output_root.resolve(strict=True)
    if not parent.is_dir(): parser.error('Output root must be an existing directory')
    if sys.platform not in ['win32','linux']:
        print(json.dumps({'status':'Unsupported','reason':'This guest collector supports Windows/Linux only'}))
        return 2
    fs = filesystem(parent)
    if not fs['native_work_root']:
        print(json.dumps({'status':'Blocked','reason':'Guest-local native filesystem not proven','filesystem':fs}))
        return 2
    output = parent / ('guest-preflight-'+datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')+'-'+uuid.uuid4().hex[:8])
    output.mkdir()
    environment = {k:v for k,v in os.environ.items() if k != 'PIVOT_DESKTOP_PERMIT' and not k.startswith('PIVOT_UI_')}
    environment.update({'DOTNET_CLI_TELEMETRY_OPTOUT':'1','DOTNET_SKIP_FIRST_TIME_EXPERIENCE':'1',
                        'DOTNET_GENERATE_ASPNET_CERTIFICATE':'false',
                        'DOTNET_CLI_HOME':str(output/'dotnet-home'),'DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE':'true',
                        'RUSTUP_AUTO_INSTALL':'0'})
    checks = {'rust':run(['rustc','-vV'],cwd=output,env=environment),
              'cargo':run(['cargo','-V'],cwd=output,env=environment),
              'dotnet':run(['dotnet','--info'],cwd=output,env=environment)}
    if os.name == 'nt':
        checks['host-session'] = run(['powershell','-NoProfile','-NonInteractive','-Command',
            "[pscustomobject]@{SessionId=[System.Diagnostics.Process]::GetCurrentProcess().SessionId;Computer=(Get-CimInstance Win32_ComputerSystem | Select-Object Manufacturer,Model,HypervisorPresent);OS=(Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,LastBootUpTime)} | ConvertTo-Json -Depth 4"])
        # cl may require an explicitly prepared native developer shell. Missing is evidence, not an installation request.
        checks['native-linker'] = run(['where.exe','link.exe'])
        checks['compiler-location'] = run(['where.exe','cl.exe'])
        checks['linker-version'] = run(['link.exe','/?'],cwd=output,env=environment)
        checks['compiler-version'] = run(['cl.exe','/?'],cwd=output,env=environment)
        for key in ['native-linker','compiler-location']:
            if checks[key].get('exit_code') == 1:checks[key]['status'] = 'NotConfigured'
    else:
        checks['host-session'] = run(['systemd-detect-virt'])
        checks['native-linker'] = run(['cc','--version'])
        checks['clang'] = run(['clang','--version'])
        checks['linker-version'] = run(['ld','--version'])
        checks['os-release'] = {'status':'Verified','text':Path('/etc/os-release').read_text()}
        checks['boot-id'] = {'status':'Verified','text':Path('/proc/sys/kernel/random/boot_id').read_text().strip()}
    native_arch = kernel_arch(); python_arch = image_arch(Path(sys.executable).resolve())
    process_arch = windows_architectures()[1] if os.name == 'nt' else python_arch
    images = {'python':{'path':sys.executable,'architecture':python_arch}}
    for name in ['rustc','dotnet']:
        path = shutil.which(name)
        images[name] = {'path':path,'architecture':image_arch(Path(path).resolve()) if path else 'Unknown'}
    expected_host = {('win32','x64'):'x86_64-pc-windows-msvc',('win32','arm64'):'aarch64-pc-windows-msvc',
                     ('linux','x64'):'x86_64-unknown-linux-gnu',('linux','arm64'):'aarch64-unknown-linux-gnu'}.get((sys.platform,native_arch))
    if checks['rust']['status'] == 'Verified' and ('host: '+str(expected_host)) not in checks['rust']['stdout']:
        checks['rust']['status'] = 'Blocked';checks['rust']['qualification_reason'] = 'Expected native Rust host/ABI not proven'
    if any(images[name]['architecture'] != native_arch for name in ['rustc','dotnet']):
        checks['native-images'] = {'status':'Blocked','reason':'Native compiler/runtime image architecture not proven'}
    else:checks['native-images'] = {'status':'Verified'}
    probes = {}
    if args.probe:
        rust = output/'rust'; rust.mkdir(); exe = rust/('probe.exe' if os.name == 'nt' else 'probe')
        (rust/'probe.rs').write_text('fn main(){println!("PIVOT_PREFLIGHT {} {} {}",std::env::consts::OS,std::env::consts::ARCH,std::mem::size_of::<usize>()*8);}\n')
        probes['rust-build'] = run(['rustc',str(rust/'probe.rs'),'-o',str(exe)],env=environment)
        if probes['rust-build']['status'] == 'Verified':
            probes['rust-execute'] = run([str(exe)],env=environment)
            probes['rust-execute']['image_architecture'] = image_arch(exe)
            target_arch = {'x64':'x86_64','arm64':'aarch64'}.get(native_arch)
            target_os = 'windows' if os.name == 'nt' else 'linux'
            if (image_arch(exe) != native_arch or probes['rust-execute'].get('stdout','').strip()
                    != 'PIVOT_PREFLIGHT '+target_os+' '+str(target_arch)+' 64'):
                probes['rust-execute']['status'] = 'Failed'
        dotnet = output/'dotnet'; dotnet.mkdir()
        (dotnet/'NuGet.Config').write_text('<configuration><packageSources><clear /></packageSources></configuration>\n')
        (dotnet/'global.json').write_text('{"sdk":{"version":"10.0.100","rollForward":"latestFeature","allowPrerelease":false}}\n')
        (dotnet/'Probe.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>\n')
        (dotnet/'Program.cs').write_text('System.Console.WriteLine("PIVOT_PREFLIGHT "+System.Runtime.InteropServices.RuntimeInformation.OSArchitecture+" "+System.Runtime.InteropServices.RuntimeInformation.ProcessArchitecture);\n')
        probes['dotnet-build'] = run(['dotnet','build',str(dotnet/'Probe.csproj'),'-c','Release','--disable-build-servers','--configfile',str(dotnet/'NuGet.Config')],cwd=dotnet,env=environment)
        if probes['dotnet-build']['status'] == 'Verified':
            probes['dotnet-execute'] = run(['dotnet',str(dotnet/'bin/Release/net10.0/Probe.dll')],cwd=dotnet,env=environment)
            want = {'x64':'X64 X64','arm64':'Arm64 Arm64'}.get(native_arch)
            if not want or probes['dotnet-execute'].get('stdout','').strip() != 'PIVOT_PREFLIGHT '+want:
                probes['dotnet-execute']['status'] = 'Failed'
    candidate = capability(checks,native_arch,process_arch,True,probes)
    if python_arch != 'Unknown' and python_arch != process_arch:candidate = 'Blocked'
    if checks['native-images']['status'] != 'Verified':candidate = 'Blocked'
    result = {'schema':'pivot-guest-qualification-preflight-v1','class':'headless','collector_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'hostname':platform.node(),'user':getpass.getuser(),'platform':platform.platform(),'native_os_architecture':native_arch,'python_process_architecture':process_arch,'images':images,
        'filesystem':fs,'report':str(output),'checks':checks,'probes':probes,
        'graphical_session':{k:os.environ.get(k) for k in ['XDG_SESSION_ID','XDG_SESSION_TYPE','DISPLAY','WAYLAND_DISPLAY','SESSIONNAME']},
        'installed':'Unverified: bind intended VM to host receipt','headless_capability_candidate':candidate,
        'headless_qualified':'NotConfigured: pending independent evidence review','gui_isolation_approved':'NotConfigured',
        'limitations':['No Pivot CI or GUI acceptance run','No host integration or clipboard inspection','No remote execution or installation','No approval token issued']}
    (output/'guest.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    rows=[{'path':p.relative_to(output).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(output.rglob('*')) if p.is_file()]
    (output/'origin-hashes.json').write_text(json.dumps(rows,indent=2)+'\n',encoding='utf-8')
    print(str(output));print(json.dumps({'headless_capability_candidate':candidate,'gui_isolation_approved':'NotConfigured'}))
    return 0 if candidate == 'Verified' else 2


if __name__ == '__main__':raise SystemExit(main())
