"""Evidence wrapper checks only: no actual CI, guest process, GUI or clipboard."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/qualification'))
spec=importlib.util.spec_from_file_location('guest_ci',ROOT/'scripts/qualification/run_headless_ci.py')
worker=importlib.util.module_from_spec(spec);spec.loader.exec_module(worker)


class BundleTests(unittest.TestCase):
    def test_whole_wrapper_preserves_fake_ci_pass_and_failure_without_real_ci(self):
        # The only child is this generated console stub, never Pivot or Avalonia.
        for exit_code in [0,1]:
            with self.subTest(exit_code=exit_code), tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);bundle=root/'bundle';bundle.mkdir();work=root/'work';work.mkdir()
                stub=('import argparse,json,pathlib,sys\n'
                      'p=argparse.ArgumentParser();p.add_argument("--output-dir");a=p.parse_args()\n'
                      'out=pathlib.Path(a.output_dir)/"stub";out.mkdir(parents=True)\n'
                      'data={"status":"'+('passed' if exit_code==0 else 'failed')+'","desktop":False,"steps":[{"exit_code":'+str(exit_code)+'}]*13}\n'
                      '(out/"summary.json").write_text(json.dumps(data))\n'
                      'sys.exit('+str(exit_code)+')\n').encode()
                rows=[{'path':'scripts/ci.py','sha256':hashlib.sha256(stub).hexdigest()}]
                with zipfile.ZipFile(bundle/'source.zip','w') as packed:packed.writestr('scripts/ci.py',stub)
                for name in ['source-manifest.json','build-input-manifest.json']:(bundle/name).write_text(json.dumps(rows))
                (bundle/'run_headless_ci.py').write_bytes((ROOT/'scripts/qualification/run_headless_ci.py').read_bytes())
                receipt={'files':[{'path':p.name,'sha256':worker.sha(p)} for p in bundle.iterdir()]}
                (bundle/'bundle-receipt.json').write_text(json.dumps(receipt))
                args=['worker','--bundle-dir',str(bundle),'--work-root',str(work),
                      '--expected-hostname','SyntheticGuest','--return-root',str(root/'return')]
                with patch('sys.argv',args),patch.object(worker,'filesystem',return_value={'native_work_root':True}), \
                        patch.object(worker,'kernel_arch',return_value='x64'), \
                        patch.object(worker,'windows_architectures',return_value=('x64','x64')), \
                        patch.object(worker,'image_arch',return_value='x64'), \
                        patch.object(worker.platform,'node',return_value='SyntheticGuest'), \
                        patch.object(worker,'owned_processes',return_value={'status':'Verified','matches':[]}):
                    self.assertEqual(worker.main(),exit_code)
                results=list((root/'return').glob('*/result.json'));self.assertEqual(len(results),1)
                result=json.loads(results[0].read_text());self.assertEqual(result['ci_exit_code'],exit_code)
                self.assertEqual(result['status'],'Passed' if exit_code==0 else 'Failed')
                hashes=json.loads((results[0].parent/'origin-hashes.json').read_text())
                worker.verify_files(results[0].parent,hashes)

    def test_verified_extract_and_hash_mismatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);archive=root/'source.zip';destination=root/'source';destination.mkdir()
            with zipfile.ZipFile(archive,'w') as packed:packed.writestr('src/test.py',b'print(1)\n')
            rows=[{'path':'src/test.py','sha256':hashlib.sha256(b'print(1)\n').hexdigest()}]
            worker.unpack_verified(archive,destination,rows)
            self.assertEqual(worker.same_source(destination,rows),[])
            (destination/'src/test.py').write_text('changed')
            self.assertEqual(worker.same_source(destination,rows),['src/test.py'])
            with self.assertRaises(ValueError):worker.verify_files(destination,rows)

    def test_unsafe_inventory_refuses_before_any_file_write(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);destination=root/'source';destination.mkdir()
            for name in ['../escape','/absolute','C:/escape','dir\\escape']:
                archive=root/'source.zip'
                with zipfile.ZipFile(archive,'w') as packed:packed.writestr(name,b'x')
                with patch.object(Path,'open',side_effect=AssertionError('File opened before unsafe inventory rejection')), self.assertRaises(ValueError):
                    worker.unpack_verified(archive,destination,[{'path':name,'sha256':hashlib.sha256(b'x').hexdigest()}])
                self.assertEqual(list(destination.rglob('*')),[])

    def test_extra_missing_and_duplicate_archive_entries_refuse(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);destination=root/'source';destination.mkdir();archive=root/'source.zip'
            with zipfile.ZipFile(archive,'w') as packed:packed.writestr('extra',b'x')
            with self.assertRaises(ValueError):worker.unpack_verified(archive,destination,[])
            with zipfile.ZipFile(archive,'w'):pass
            row={'path':'missing','sha256':'0'*64}
            with self.assertRaises(ValueError):worker.unpack_verified(archive,destination,[row])
            with self.assertRaises(ValueError):worker.unpack_verified(archive,destination,[row,row])

    def test_build_outputs_allowed_but_added_source_reported(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);(root/'obj').mkdir();(root/'obj/generated.cs').write_text('generated')
            self.assertEqual(worker.same_source(root,[]),[])
            (root/'added.py').write_text('unexpected input')
            self.assertEqual(worker.same_source(root,[]),['ADDED: added.py'])


if __name__=='__main__':unittest.main()
