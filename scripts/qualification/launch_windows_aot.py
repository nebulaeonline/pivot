"""Guest-local Native AOT evidence bootstrap. Positive path remains disabled."""
from pathlib import Path
import shutil
import sys
import traceback
import uuid

from launch_windows_managed import Tee,collect_native_report,capture_status
from run_headless_ci import read_json,sha,verify_files,write_json


def main():
    if sys.platform!='win32':raise SystemExit('Windows guest only')
    if len(sys.argv) not in (2,3) or (len(sys.argv)==3 and sys.argv[2]!='--run-aot'):
        raise SystemExit('Usage: launch_windows_aot.py RECEIPT [--run-aot]')
    digest=sys.argv[1];positive=len(sys.argv)==3;bundle=Path(__file__).resolve().parent
    if sha(bundle/'bundle-receipt.json')!=digest:raise SystemExit('Receipt mismatch; no runner imported')
    verify_files(bundle,read_json(bundle/'bundle-receipt.json')['files'])
    import windows_attended_aot as worker
    parent=worker.common.local_tree(Path('C:/PivotWork'))
    report=parent/('aot-capture-'+uuid.uuid4().hex);report.mkdir()
    oldout,olderr=sys.stdout,sys.stderr;code=2
    with (report/'console.log').open('w',encoding='utf-8') as log:
        try:
            sys.stdout,sys.stderr=Tee(oldout,log),Tee(olderr,log)
            sys.argv=['windows_attended_aot.py','--bundle-dir',str(bundle),'--receipt-sha256',digest,'--work-root',str(parent)]
            if positive:sys.argv.append('--run-aot')
            code=worker.main()
        except (Exception,KeyboardInterrupt):traceback.print_exc()
        finally:sys.stdout,sys.stderr=oldout,olderr
    errors=collect_native_report((report/'console.log').read_text(),code,parent,report,worker.common)
    run_path=report/'native-report/aot-run.json'
    run=read_json(run_path) if run_path.is_file() else None
    status=capture_status(run,code,errors)
    for name in ['source-manifest.json','build-input-manifest.json','bundle-receipt.json']:shutil.copy2(bundle/name,report/name)
    write_json(report/'capture.json',{'status':status,'exit_code':code,'GUI_executed':None if run else False,
               'fixed_entry':'scripts/verify_native_aot.py','export_errors':errors,
               'packet_receipt_sha256':digest,'mode':'attended Native AOT' if positive else 'non-GUI AOT preparation'})
    rows=[{'path':p.relative_to(report).as_posix(),'sha256':sha(p)} for p in sorted(report.rglob('*')) if p.is_file()]
    write_json(report/'origin-hashes.json',rows)
    destination=Path('J:/coding/pivot/artifacts/windows-native-aot/results/Pivot-Win11-x64')/report.name
    try:
        destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(report,destination);verify_files(destination,rows)
        write_json(destination/'transfer-receipt.json',{'status':'Verified','native_manifest_sha256':sha(report/'origin-hashes.json'),
                   'returned_manifest_sha256':sha(destination/'origin-hashes.json')})
        print('Returned: '+str(destination))
    except (OSError,ValueError) as error:
        print('Transfer failed; retain native evidence, do not replay: '+str(report)+'; '+str(error));return 2
    return 2 if errors and code==0 else code


if __name__=='__main__':raise SystemExit(main())
