"""Non-GUI experiment plumbing: PID identity, WMI metadata and target instrumentation."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/qualification'))
import collector_build_probe as probe


class CollectorProbeTests(unittest.TestCase):
    def test_pid_reuse_and_preexisting_vs_current_build(self):
        work=Path('C:/PivotWork/collector-build-test')
        old={'pid':10,'start_ticks':'100','command':'C:/PivotWork/attended-old/Avalonia.BuildServices.Collector.dll'}
        new={'pid':10,'start_ticks':'200','command':str(work)+'/Avalonia.BuildServices.Collector.dll'}
        records=[{'kind':'snapshot','phase':'before-build','processes':[old]},
                 {'kind':'creation-observed','phase':'build','processes':[new]},
                 {'kind':'snapshot','phase':'immediate-after-build','processes':[new]},
                 {'kind':'snapshot','phase':'bounded-observation','processes':[]}]
        result=probe.process_summary(records,work)
        self.assertEqual(result['preexisting_collectors'],[old])
        self.assertEqual(result['current_build_collectors_observed'],[new])
        self.assertEqual(result['current_build_creation_events'],[[10,'200']])
        self.assertEqual(result['current_build_immediate_presence'],[[10,'200']])
        self.assertEqual(result['current_build_final_presence'],[])
        records[-1]['processes']=[new]
        self.assertEqual(probe.process_summary(records,work)['current_build_final_presence'],[[10,'200']])

    def test_missing_process_start_identity_does_not_pass(self):
        with self.assertRaises(ValueError):probe.process_summary([],Path('.'))
        with self.assertRaises(ValueError):probe.process_summary([{'processes':[{'pid':1}]}],Path('.'))

    @unittest.skipUnless(sys.platform=='win32','Windows WMI observer')
    def test_actual_read_only_observer_baseline_and_stop(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            env=dict(os.environ,PIVOT_COLLECTOR_REPORT=str(root),PIVOT_COLLECTOR_WORK='C:/PivotWork/collector-build-probe')
            with (root/'console.log').open('w') as log:
                process=subprocess.Popen(['powershell.exe','-NoProfile','-NonInteractive','-Command',
                                           (ROOT/'scripts/qualification/collector_monitor.ps1').read_text()],
                                          env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT)
                try:
                    deadline=time.monotonic()+20
                    while not (root/'ready.json').exists() and process.poll() is None and time.monotonic()<deadline:time.sleep(.1)
                    self.assertTrue((root/'ready.json').exists(),(root/'console.log').read_text())
                    self.assertEqual(json.loads((root/'ready.json').read_text())['status'],'Ready')
                finally:
                    (root/'stop').write_text('stop');process.wait(timeout=15)
            self.assertEqual(json.loads((root/'monitor-result.json').read_text())['status'],'Completed')
            rows=[json.loads(line) for line in (root/'processes.ndjson').read_text(encoding='utf-8-sig').splitlines()]
            self.assertTrue(rows);self.assertEqual(rows[0]['kind'],'snapshot')

    def test_actual_msbuild_targets_record_counts_without_license_contents(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);out=root/'evidence';out.mkdir()
            (root/'global.json').write_bytes((ROOT/'global.json').read_bytes())
            (root/'NuGet.Config').write_text('<configuration><packageSources><clear /></packageSources></configuration>')
            (root/'observe.targets').write_text(probe.EVIDENCE_TARGETS)
            (root/'Synthetic.csproj').write_text('''<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup>
<TargetFramework>net10.0</TargetFramework></PropertyGroup>
<Target Name="AvaloniaStats" BeforeTargets="CoreCompile"><Message Text="Synthetic non-GUI target only" /></Target></Project>''')
            (root/'Class.cs').write_text('public sealed class Example {}')
            env=dict(os.environ,AVALONIA_TELEMETRY_OPTOUT='1',DOTNET_GENERATE_ASPNET_CERTIFICATE='false',DOTNET_CLI_TELEMETRY_OPTOUT='1')
            build=subprocess.run(['dotnet','build',str(root/'Synthetic.csproj'),'--disable-build-servers','-warnaserror',
                                  '-p:CustomAfterMicrosoftCommonTargets='+str(root/'observe.targets'),
                                  '-p:PivotCollectorEvidenceRoot='+str(out)],cwd=root,env=env,
                                 capture_output=True,text=True,timeout=120)
            self.assertEqual(build.returncode,0,build.stdout+build.stderr)
            files=list(out.glob('*.txt'));self.assertEqual(len(files),2)
            for path in files:
                text=path.read_text(encoding='utf-8-sig').strip()
                self.assertIn('license_items=0',text);self.assertIn('optout=1',text)

    @unittest.skipUnless(sys.platform=='win32','Windows WMI process creation')
    def test_observer_sees_known_console_fixture_creation_and_start_identity(self):
        # A three-second console double, not the real collector or Avalonia.
        parent=Path('C:/PivotWork');parent.mkdir(exist_ok=True);probe.wrapper.local_tree(parent)
        with tempfile.TemporaryDirectory(prefix='collector-build-observer-',dir=parent) as tmp:
            root=Path(tmp);report=root/'report';report.mkdir()
            (root/'global.json').write_bytes((ROOT/'global.json').read_bytes())
            (root/'NuGet.Config').write_text('<configuration><packageSources><clear /></packageSources></configuration>')
            (root/'Fixture.csproj').write_text('''<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup>
<TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
<AssemblyName>Avalonia.BuildServices.Collector</AssemblyName></PropertyGroup></Project>''')
            (root/'Program.cs').write_text('System.Console.WriteLine("Synthetic observer fixture only"); System.Threading.Thread.Sleep(3000);')
            env=dict(os.environ,DOTNET_GENERATE_ASPNET_CERTIFICATE='false',DOTNET_CLI_TELEMETRY_OPTOUT='1',
                     PIVOT_COLLECTOR_REPORT=str(report),PIVOT_COLLECTOR_WORK=str(root))
            build=subprocess.run(['dotnet','build',str(root/'Fixture.csproj'),'--disable-build-servers','-warnaserror'],
                                 cwd=root,env=env,capture_output=True,text=True,timeout=120)
            self.assertEqual(build.returncode,0,build.stdout+build.stderr)
            with (report/'monitor-console.log').open('w') as log:
                observer=subprocess.Popen(['powershell.exe','-NoProfile','-NonInteractive','-Command',
                                           (ROOT/'scripts/qualification/collector_monitor.ps1').read_text()],
                                          env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT)
                try:
                    deadline=time.monotonic()+20
                    while not (report/'ready.json').exists() and observer.poll() is None and time.monotonic()<deadline:time.sleep(.1)
                    self.assertEqual(json.loads((report/'ready.json').read_text())['status'],'Ready')
                    fixture=subprocess.run(['dotnet',str(root/'bin/Debug/net10.0/Avalonia.BuildServices.Collector.dll')],
                                           cwd=root,env=env,capture_output=True,text=True,timeout=15)
                    self.assertEqual(fixture.returncode,0,fixture.stdout+fixture.stderr)
                    time.sleep(1.5)
                finally:
                    (report/'stop').write_text('stop');observer.wait(timeout=15)
            self.assertEqual(json.loads((report/'monitor-result.json').read_text())['status'],'Completed')
            records=[json.loads(line) for line in (report/'processes.ndjson').read_text(encoding='utf-8-sig').splitlines()]
            creations=[r for e in records if e['kind']=='creation-observed' for r in e['processes']
                       if str(root).lower() in r['command'].lower()]
            snapshots=[r for e in records if e['kind']=='snapshot' for r in e['processes']
                       if str(root).lower() in r['command'].lower()]
            self.assertTrue(creations,records)
            self.assertTrue(any((c['pid'],c['start_ticks'])==(s['pid'],s['start_ticks']) for c in creations for s in snapshots))


if __name__=='__main__':unittest.main()
