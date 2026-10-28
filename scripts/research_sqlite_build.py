"""Build private instrumentation plus the pinned official amalgamation, locally."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def build(vendor,output):
    output.mkdir(parents=True,exist_ok=True);root=Path(__file__).resolve().parent
    pin=json.loads((root/'research/sqlite-source.json').read_text())
    actual=hashlib.sha3_256((vendor/'sqlite3.c').read_bytes()).hexdigest()
    if actual!=pin['sqlite3_c_sha3_256']:raise ValueError('amalgamation source checksum mismatch')
    source=root/'research/sqlite_bridge.c'
    library=output/('pivot_sqlite.dll' if os.name=='nt' else ('libpivot_sqlite.dylib' if sys.platform=='darwin' else 'libpivot_sqlite.so'))
    if os.name=='nt':
        where=Path(os.environ.get('ProgramFiles(x86)','C:/Program Files (x86)'))/'Microsoft Visual Studio/Installer/vswhere.exe'
        install=subprocess.check_output([str(where),'-latest','-products','*','-requires','Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-property','installationPath'],text=True).strip()
        if not install:raise RuntimeError('native MSVC x64 tools required')
        command=output/'build.cmd';command.write_text('@call "'+str(Path(install)/'VC/Auxiliary/Build/vcvars64.bat')+'"\n@if errorlevel 1 exit /b %errorlevel%\ncl /nologo /O2 /LD /I"'+str(vendor)+'" "'+str(source)+'" /Fe:"'+str(library)+'"\n',newline='\r\n')
        argv=['cmd.exe','/d','/c',str(command)]
    else:
        argv=['clang' if sys.platform=='darwin' else 'cc','-O2','-fPIC','-std=c11','-D_GNU_SOURCE',
              '-dynamiclib' if sys.platform=='darwin' else '-shared','-I'+str(vendor),str(source),'-o',str(library),'-lpthread','-lm']
        if sys.platform!='darwin':argv+=['-ldl']
    process=subprocess.run(argv,cwd=output,capture_output=True,text=True)
    (output/'compile.log').write_text(process.stdout+process.stderr)
    if process.returncode:raise RuntimeError('native compiler failed; see compile.log')
    report={'pin':pin,'command':argv,'bridge_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
            'library_sha256':hashlib.sha256(library.read_bytes()).hexdigest()}
    (output/'build.json').write_text(json.dumps(report,indent=2));return library


if __name__=='__main__':print(build(Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve()))
