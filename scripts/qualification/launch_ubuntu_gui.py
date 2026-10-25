"""Fixed attended terminal recorder; positive entry remains hard-disabled."""
import argparse
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import uuid

import ubuntu_attended_gui as contract
import ubuntu_gui_runtime as runtime

def native_report(text,root):
 paths=[line.split('Native report: ',1)[1].strip() for line in text.splitlines() if 'Native report: ' in line]
 if len(paths)!=1:raise ValueError('Exactly one original native report required')
 path=Path(paths[0]);runtime.local_tree(path)
 if path.name!='report' or path.parent.parent!=root:raise ValueError('Unexpected native report path')
 return path

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--packet-dir',type=Path,required=True);parser.add_argument('--receipt-sha256',required=True)
 parser.add_argument('--run-managed',action='store_true');args=parser.parse_args()
 if args.run_managed and not contract.POSITIVE_RUN_APPROVED:
  print('Blocked: Ubuntu positive GUI disabled.',file=sys.stderr);return 2
 if not sys.stdin.isatty() or not sys.stdout.isatty():raise ValueError('Original interactive terminal required')
 packet=runtime.local_tree(args.packet_dir)
 if runtime.sha(packet/'bundle-receipt.json')!=args.receipt_sha256:raise ValueError('Pinned receipt mismatch')
 runtime.verify_files(packet,runtime.read_json(packet/'bundle-receipt.json')['files'])
 for name in ['launch_ubuntu_gui.py','ubuntu_attended_gui.py','ubuntu_gui_runtime.py','ubuntu_gui_preparation.py','guest_preflight.py','run_headless_ci.py']:
  if runtime.sha(Path(__file__).parent/name)!=runtime.sha(packet/name):raise ValueError('Launcher/helper differs from packet')
 recorder=shutil.which('script')
 if not recorder:raise ValueError('Installed util-linux script recorder unavailable; no install attempted')
 parent=runtime.local_tree(Path.home()/'pivot-preflight');parent.mkdir(exist_ok=True)
 capture=parent/('ubuntu-capture-'+uuid.uuid4().hex);capture.mkdir();report=capture/'report';report.mkdir()
 command=[sys.executable,str(packet/'ubuntu_attended_gui.py'),'--packet-dir',str(packet),'--receipt-sha256',args.receipt_sha256]
 if args.run_managed:command.append('--run-managed')
 # util-linux script supplies an actual PTY and records native subprocess output.
 # No input recording/response injection; original consent remains interactive.
 code=subprocess.call([recorder,'--quiet','--return','--flush','--command',shlex.join(command),str(report/'console.log')])
 errors=[]
 try:
  native=native_report((report/'console.log').read_text(errors='replace'),parent)
  runtime.verify_files(native,runtime.read_json(native/'origin-hashes.json'))
  shutil.copytree(native,report/'native-report')
  runtime.verify_files(report/'native-report',runtime.read_json(native/'origin-hashes.json'))
 except (OSError,ValueError,KeyError) as error:errors.append(str(error))
 runtime.write_json(report/'capture.json',{'runner_exit_code':code,'status':runtime.classify(code,errors),
  'errors':errors,'packet_receipt_sha256':args.receipt_sha256,'mode':'managed' if args.run_managed else 'non-GUI preflight',
  'GUI_executed':None if args.run_managed else False,'recorder':'util-linux script; no supplied consent'})
 print('Native capture: '+str(report),flush=True)
 try:runtime.export(report,Path('/mnt/j/coding/pivot/artifacts/ubuntu-managed-implementation/captures/Pivot-Ubuntu-x64'))
 except (OSError,ValueError) as error:
  print('Export failed; retain native capture, copy-only recovery: '+str(error),file=sys.stderr)
  if code==0:code=2
 return 2 if code==0 and errors else code

if __name__=='__main__':
 try:raise SystemExit(main())
 except (OSError,ValueError,subprocess.SubprocessError) as error:
  print('Blocked: '+str(error),file=sys.stderr);raise SystemExit(2)
