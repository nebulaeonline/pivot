"""Compile the actual capture/Closed report fragment with a non-GUI window double.

No Avalonia dependency, windows, clipboard, GUI consent or positive suite.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts/qualification'))
import windows_attended_gui as wrapper


class PeerBackendLifecycle(unittest.TestCase):
    def test_actual_live_capture_and_closed_serialization_without_avalonia(self):
        source = (ROOT/'clients/dotnet/Pivot.Desktop/ClipboardAutomation.cs').read_text()
        method = source.split('private async Task ClipboardPeerAsync(', 1)[1]
        capture = method.index('string? peerGraphicalBackend=')
        close = method.index('\n        Close();', capture)
        fragment = method[capture:close]
        self.assertLess(fragment.index('PlatformImpl?.GetType().FullName'), fragment.index('Closed+='))
        self.assertNotIn('PlatformImpl', fragment.split('Closed+=', 1)[1])
        with tempfile.TemporaryDirectory(prefix='pivot-peer-backend-') as tmp:
            root = Path(tmp)
            project = ('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup>'
                       '<OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework>'
                       '<Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>'
                       '<JsonSerializerIsReflectionEnabledByDefault>false</JsonSerializerIsReflectionEnabledByDefault>'
                       '</PropertyGroup><ItemGroup><Compile Include="'+escape(str(ROOT/'clients/dotnet/Pivot.Desktop/AutomationReports.cs'))+'" Link="AutomationReports.cs" />'
                       '</ItemGroup></Project>')
            (root/'Probe.csproj').write_text(project)
            (root/'global.json').write_bytes((ROOT/'global.json').read_bytes())
            (root/'NuGet.Config').write_text('<configuration><packageSources><clear /></packageSources></configuration>')
            program = '''using System.Text.Json;
using Pivot.Desktop;
var window=new FakeWindow();
window.PlatformImpl=args[1]=="live"?new ProbeBackend():null;
window.Run(args[0]);
if(window.PlatformImpl is not null) throw new Exception("Close did not clear platform");
sealed class ProbeBackend {}
sealed class Session { public ulong FinalCharge=0; public int ProcessId=123; }
sealed class FakeWindow {
 public object? PlatformImpl;
 public event EventHandler? Closed;
 private readonly Session session=new();
 private readonly List<object> OwnedWindows=new();
 public void Run(string report) {
 string mode="clean",path="scratch";bool cancelled=false,nativeReceived=false;
''' + fragment + '''
 Close();
 }
 private void Close(){ PlatformImpl=null; Closed?.Invoke(this,EventArgs.Empty); }
}
'''
            (root/'Program.cs').write_text(program)
            env = {k: v for k, v in os.environ.items() if not k.upper().startswith('PIVOT_')}
            env.update(DOTNET_GENERATE_ASPNET_CERTIFICATE='false', DOTNET_CLI_TELEMETRY_OPTOUT='1')
            build = subprocess.run(['dotnet', 'build', str(root/'Probe.csproj'), '-c', 'Release',
                                    '--disable-build-servers', '-warnaserror'], cwd=root, env=env,
                                   capture_output=True, text=True, timeout=120)
            self.assertEqual(build.returncode, 0, build.stdout+build.stderr)
            for mode in ['live', 'missing']:
                report = root/(mode+'.json')
                run = subprocess.run(['dotnet', str(root/'bin/Release/net10.0/Probe.dll'), str(report), mode],
                                     cwd=root, env=env, capture_output=True, text=True, timeout=30)
                self.assertEqual(run.returncode, 0, run.stdout+run.stderr)
                data = json.loads(report.read_text())
                self.assertEqual(data['remaining_charge'], 0)
                self.assertEqual(data['status'], 'passed')  # unchanged functional/cleanup classification
                with patch.object(wrapper, 'read_json', return_value=data), \
                        patch.object(wrapper, 'desktop_identity', return_value={'session_id': data['process_session_id']}):
                    if mode == 'live':
                        self.assertEqual(data['graphical_backend'], 'ProbeBackend')
                        self.assertEqual(wrapper.managed_evidence(root)['application_reports'], 4)
                    else:
                        self.assertNotIn('graphical_backend', data)
                        with self.assertRaises(ValueError): wrapper.managed_evidence(root)


if __name__ == '__main__': unittest.main()
