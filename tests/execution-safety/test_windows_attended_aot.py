"""Native AOT preparation negatives and artifact contracts. Never publish/run GUI."""
from copy import deepcopy
import contextlib
import io
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'scripts/qualification'),str(ROOT/'tests/execution-safety')]
import windows_attended_aot as worker
from test_windows_attended_gui import reviewed_fixture
from run_headless_ci import verify_files


def pe(path,machine=0x8664):
    path.parent.mkdir(parents=True,exist_ok=True)
    payload=bytearray(70);payload[:2]=b'MZ';struct.pack_into('<I',payload,60,64)
    payload[64:68]=b'PE\0\0';struct.pack_into('<H',payload,68,machine);path.write_bytes(payload)


class AotPreparationTests(unittest.TestCase):
    def test_no_managed_fallback_arbitrary_command_or_rid_selector(self):
        for option in ['--run-managed','--command','--rid']:
            args=['aot','--bundle-dir','.','--receipt-sha256','fixture','--work-root','.',option]
            with self.subTest(option=option),patch.object(sys,'argv',args),patch.object(worker.common,'main') as preflight, \
                    contextlib.redirect_stderr(io.StringIO()),self.assertRaises(SystemExit) as result:
                worker.main()
            self.assertEqual(result.exception.code,2);preflight.assert_not_called()

    def test_positive_gate_refuses_before_preflight_tools_or_runner(self):
        args=['aot','--bundle-dir','.','--receipt-sha256','0'*64,'--work-root','.','--run-aot']
        with patch.object(sys,'argv',args),patch.object(worker.common,'main') as preflight, \
                patch.object(worker,'prerequisites') as tools,contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(worker.main(),2);preflight.assert_not_called();tools.assert_not_called()
        with self.assertRaises(ValueError):worker.invoke_aot(Path('.'),Path('.'),{})
        with self.assertRaises(ValueError):worker.run_fixed_aot(Path('.'),Path('.'),{},reviewed_fixture(),[])

    def test_native_compiler_survivor_scope_and_missing_start_identity(self):
        with patch.object(worker.subprocess,'run') as query:
            row={'ProcessId':123,'Name':'link.exe','CommandLine':'link output','StartTicks':'12345'}
            query.return_value=SimpleNamespace(returncode=0,stdout=json.dumps([row]),stderr='')
            result=worker.owned_processes(Path('C:/PivotWork/attended-test'))
            self.assertEqual(result['status'],'Verified');self.assertEqual(result['matches'],[row])
            script=query.call_args.args[0][-1]
            self.assertIn('|cl|link|rc|ilc',script)
            self.assertEqual(query.call_args.kwargs['stdin'],subprocess.DEVNULL)
            del row['StartTicks'];query.return_value.stdout=json.dumps([row])
            self.assertEqual(worker.owned_processes(Path('.'))['status'],'Blocked')

    def test_original_aot_runner_noninteractive_refusal_before_any_output_or_publish(self):
        with tempfile.TemporaryDirectory() as tmp:
            out=Path(tmp)/'never-created'
            env={k:v for k,v in os.environ.items() if not k.upper().startswith('PIVOT_')}
            run=subprocess.run([sys.executable,str(ROOT/'scripts/verify_native_aot.py'),'--output-dir',str(out)],
                               cwd=ROOT,env=env,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=30)
            self.assertEqual(run.returncode,2,run.stdout+run.stderr)
            self.assertIn('Blocked',run.stderr);self.assertFalse(out.exists())

    def test_fixed_invocation_keeps_real_streams_and_restores_caller(self):
        with tempfile.TemporaryDirectory() as tmp,patch.object(worker,'AOT_EXECUTION_APPROVED',True), \
                patch.object(worker.runpy,'run_path') as entry:
            source=Path(tmp);original=(dict(os.environ),sys.argv[:],sys.path[:],sys.stdin,sys.stdout)
            def body(path,run_name):
                self.assertEqual(path,str(source/worker.ENTRY));self.assertEqual(run_name,'__main__')
                self.assertEqual(sys.path[0],str(source/'scripts'))
                self.assertEqual(sys.argv,[worker.ENTRY,'--output-dir',str(source/'native-aot')])
                self.assertEqual(dict(os.environ),{'AVALONIA_TELEMETRY_OPTOUT':'1'})
                self.assertIs(sys.stdin,original[3]);self.assertIs(sys.stdout,original[4])
                raise SystemExit(2)
            entry.side_effect=body
            self.assertEqual(worker.invoke_aot(source,source,{'AVALONIA_TELEMETRY_OPTOUT':'1'}),2)
            self.assertEqual(dict(os.environ),original[0]);self.assertEqual(sys.argv,original[1]);self.assertEqual(sys.path,original[2])

    def test_prerequisite_contract_and_x86_missing_library_sdk_refusals(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);tools={}
            for name in ['dotnet','rustc','cargo','rc']:tools[name]=root/'tools'/(name+'.exe')
            for name in ['cl','link']:tools[name]=root/'vc/bin/Hostx64/x64'/(name+'.exe')
            for path in tools.values():pe(path)
            vswhere=root/'pf/Microsoft Visual Studio/Installer/vswhere.exe';pe(vswhere,0x14c)
            libraries=[root/'sdk/Lib/1/um/x64/kernel32.lib',root/'sdk/Lib/1/ucrt/x64/ucrt.lib',root/'vc/lib/x64/libcmt.lib']
            for path in libraries:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'library fixture')
            env={'PATH':'fixture','PROGRAMFILES(X86)':str(root/'pf'),'WINDOWSSDKDIR':str(root/'sdk'),
                 'WINDOWSSDKVERSION':'1\\','VCTOOLSINSTALLDIR':str(root/'vc')}
            version=['10.0.402']
            def run(command,**kwargs):
                if 'vswhere.exe' in command[0]:text='[{"installationPath":"fixture","installationVersion":"17.14"}]'
                elif command[:2]==['dotnet','--version']:text=version[0]
                elif command[:2]==['dotnet','--info']:text='RID: win-x64'
                else:text='host: x86_64-pc-windows-msvc'
                return SimpleNamespace(returncode=0,stdout=text,stderr='')
            with patch.object(worker.sys,'platform','win32'),patch.object(worker.common,'local_tree',side_effect=lambda p:p), \
                    patch.object(worker.shutil,'which',side_effect=lambda name,path=None:str(tools[name])), \
                    patch.object(worker.subprocess,'run',side_effect=run):
                self.assertEqual(worker.prerequisites(root,env)['status'],'Verified')
                pe(tools['cl'],0x14c)
                self.assertEqual(worker.prerequisites(root,env)['status'],'Blocked');pe(tools['cl'])
                version[0]='10.0.100-preview.1';self.assertEqual(worker.prerequisites(root,env)['status'],'Blocked');version[0]='10.0.402'
                libraries[0].unlink();self.assertEqual(worker.prerequisites(root,env)['status'],'Blocked')

    def test_shared_preflight_is_metadata_only_and_preserves_same_line_report_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);work=root/'work';work.mkdir();native=work/'attended-stub/report';native.mkdir(parents=True)
            (native.parent/'source').mkdir();bundle=root/'bundle';bundle.mkdir()
            (bundle/'windows_attended_aot.py').write_bytes((ROOT/'scripts/qualification/windows_attended_aot.py').read_bytes())
            worker.write_json(bundle/'bundle-receipt.json',{'files':[{'path':'windows_attended_aot.py','sha256':worker.sha(bundle/'windows_attended_aot.py')}]})
            worker.write_json(bundle/'build-input-manifest.json',[{'path':worker.ENTRY,'sha256':'fixture'}])
            def preflight():
                self.assertNotIn('--run-managed',sys.argv)
                print('Type CONFIRM: Native preflight report: '+str(native))
                return 0
            argv=['aot','--bundle-dir',str(bundle),'--receipt-sha256','fixture','--work-root',str(work)]
            with patch.object(sys,'argv',argv),patch.object(worker.common,'main',side_effect=preflight), \
                    patch.object(worker.common,'local_tree',side_effect=lambda p:p), \
                    patch.object(worker.common,'clean_environment',return_value={}), \
                    patch.object(worker,'prerequisites',return_value={'status':'Verified'}), \
                    patch.object(worker,'run_fixed_aot') as positive,contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(worker.main(),0);positive.assert_not_called()
            data=worker.read_json(native/'aot-preparation.json');self.assertFalse(data['positive_path_enabled'])
            verify_files(native,worker.read_json(native/'origin-hashes.json'))

    def fixture(self,root):
        output=root/'native-aot/one';publish=output/'publish';publish.mkdir(parents=True)
        executable=publish/'Pivot.Desktop.exe';pe(executable);pe(publish/'pivot-session.exe')
        summary={'status':'passed','rid':'win-x64','publish_warnings':[],
                 'native_desktop_and_all_peers':True,'adjacent_rust_child':True,
                 'published_files':[{'path':str(p.relative_to(publish)),'sha256':worker.sha(p)} for p in publish.iterdir()]}
        worker.write_json(output/'summary.json',summary);ui=output/'desktop-ui';ui.mkdir()
        for pid,mode in enumerate(['main','clipboard-save','discard','clean'],1):
            worker.write_json(ui/('results.json' if mode=='main' else f'results.{mode}.json'),{
                'status':'passed','remaining_charge':0,'execution_mode':'native-aot','graphical_backend':'Actual.Native.Window',
                'process_session_id':1,'process_id':pid,'executable':str(executable)})
        return output

    def test_published_image_hash_runtime_backend_session_and_charge_requirements(self):
        for scenario in ['valid','wrong-mode','missing-backend','charge','session','image-hash','managed-dll','duplicate-pid']:
            with self.subTest(scenario=scenario),tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);out=self.fixture(root);peer=out/'desktop-ui/results.clean.json';data=worker.read_json(peer)
                for value,key in [('wrong-mode','execution_mode'),('missing-backend','graphical_backend'),('charge','remaining_charge'),
                                  ('session','process_session_id'),('duplicate-pid','process_id')]:
                    if scenario==value:data[key]={'wrong-mode':'managed','missing-backend':None,'charge':1,'session':2,'duplicate-pid':1}[scenario]
                worker.write_json(peer,data)
                if scenario=='image-hash':
                    with (out/'publish/Pivot.Desktop.exe').open('ab') as stream:stream.write(b'changed')
                if scenario=='managed-dll':(out/'publish/Pivot.Desktop.dll').write_bytes(b'unexpected')
                with patch.object(worker.common,'local_tree',side_effect=lambda p:p),patch.object(worker.common,'desktop_identity',return_value={'session_id':1}):
                    if scenario=='valid':self.assertEqual(worker.aot_evidence(root)['zero_charge_fields'],4)
                    else:
                        with self.assertRaises(ValueError):worker.aot_evidence(root)

    def test_result_classification_and_strict_survivor_check_remain(self):
        for code,survivors,expected in [(0,[],'Passed'),(0,[{'pid':1}],'Blocked'),(1,[{'pid':1}],'Failed'),(2,[],'Blocked')]:
            with self.subTest(code=code,survivors=survivors),tempfile.TemporaryDirectory() as tmp, \
                    patch.object(worker,'AOT_EXECUTION_APPROVED',True), \
                    patch.object(worker.common,'guest_identity',return_value=reviewed_fixture()), \
                    patch.object(worker,'invoke_aot',return_value=code),patch.object(worker,'same_source',return_value=[]), \
                    patch.object(worker,'owned_processes',return_value={'status':'Verified','matches':survivors}), \
                    patch.object(worker,'aot_evidence',return_value={'zero_charge_fields':4}):
                result=worker.run_fixed_aot(Path(tmp),Path(tmp),{},reviewed_fixture(),[])
                self.assertEqual(worker.read_json(Path(tmp)/'aot-run.json')['status'],expected)
                self.assertEqual(result,0 if expected=='Passed' else 1 if expected=='Failed' else 2)


if __name__=='__main__':unittest.main()
