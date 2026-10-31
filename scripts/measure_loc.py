"""Count milestone source once, by physical ownership, with the installed cloc tool.

python scripts/measure_loc.py --milestone ADR-0033
No builds, platform execution, dependencies or generated output are included.
"""
import argparse
from collections import defaultdict
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCE = {'.rs', '.cs', '.fs', '.xaml', '.axaml', '.proto', '.py', '.ps1', '.psm1', '.sh', '.bash', '.bat', '.cmd', '.c', '.h', '.cpp', '.hpp', '.m', '.mm', '.js', '.ts', '.tsx', '.jsx'}
DOCS = {'.md', '.rst', '.adoc', '.txt'}
EXCLUDED = {'target', 'bin', 'obj', 'artifacts', 'node_modules', 'packages', 'vendor', 'third_party', 'third-party', '.git', '.vs', '__pycache__'}


def git(*args):
    return subprocess.check_output(['git'] + list(args), cwd=ROOT).decode('utf-8')


def count():
    return {'files': 0, 'blank': 0, 'comment': 0, 'code': 0}


def add(total, row):
    total['files'] += 1
    for key in ['blank', 'comment', 'code']:
        total[key] += row[key]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--milestone', required=True)
    args = parser.parse_args()
    if not re.fullmatch(r'[A-Za-z0-9_-]+', args.milestone):
        parser.error('Use a safe milestone identifier, such as ADR-0033')
    executable = shutil.which('cloc')
    if executable is None:
        parser.error('Install cloc and make it available on PATH; no build is needed')
    base = ROOT/'artifacts/loc'
    moment = datetime.datetime.now(datetime.timezone.utc)
    output = base/args.milestone/moment.strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True, exist_ok=False)
    files = []
    for name in sorted(set(git('ls-files', '--cached', '--others', '--exclude-standard', '-z').split('\0'))):
        if not name:
            continue
        path = ROOT/name
        if any(part.lower() in EXCLUDED for part in path.relative_to(ROOT).parts) or path.is_symlink() or not path.is_file():
            continue
        if name.lower().endswith(('.g.cs', '.g.i.cs', '.designer.cs')):
            continue
        if '\n' in name or '\r' in name:
            raise ValueError('cloc list files cannot represent a newline in a source path')
        files.append(path)
    owners = []
    for path in files:
        if path.name == 'Cargo.toml':
            text = path.read_text(encoding='utf-8')
            package = re.search(r'(?ms)^\[package\]\s*(.*?)(?=^\[|\Z)', text)
            name = re.search(r'(?m)^name\s*=\s*"([^"]+)"', package.group(1)) if package else None
            if name:
                owners.append((path.parent, name.group(1), 'Rust'))
        elif path.suffix.lower() in {'.csproj', '.fsproj', '.vcxproj'}:
            owners.append((path.parent, path.stem, '.NET' if path.suffix.lower() != '.vcxproj' else 'C++'))
    owners.sort(key=lambda item: len(item[0].parts), reverse=True)
    listing = output/'inputs.txt'
    # Forward-slash relative paths also avoid cloc 2.06's unescaped Windows
    # backslashes in per-file JSON keys. Preserve the raw tool report unchanged.
    listing.write_text(''.join(path.relative_to(ROOT).as_posix()+'\n' for path in files), encoding='utf-8')
    env = os.environ.copy()
    env['PERL_BADLANG'] = '0'
    version = subprocess.run([executable, '--version'], env=env, capture_output=True, text=True, check=True).stdout.strip()
    command = [executable, '--list-file='+str(listing), '--by-file', '--json', '--skip-uniqueness', '--force-lang=XML,axaml', '--out='+str(output/'cloc.json')]
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, check=True)
    (output/'cloc.log').write_text(result.stdout+result.stderr, encoding='utf-8')
    raw = json.loads((output/'cloc.json').read_text(encoding='utf-8-sig'))
    groups = defaultdict(count)
    projects = {}
    languages = defaultdict(count)
    digest = hashlib.sha256()
    for path in files:
        digest.update(path.relative_to(ROOT).as_posix().encode('utf-8')+b'\0'+hashlib.sha256(path.read_bytes()).digest())
    for name, row in raw.items():
        if name in ['header', 'SUM']:
            continue
        path = Path(name)
        if not path.is_absolute():
            path = ROOT/path
        relative = path.relative_to(ROOT)
        suffix = path.suffix.lower()
        if suffix in SOURCE:
            add(groups['source'], row)
            add(languages[row['language']], row)
            owner = next((item for item in owners if relative.parts[:len(item[0].relative_to(ROOT).parts)] == item[0].relative_to(ROOT).parts), None)
            if owner:
                directory, label, kind = owner
            else:
                label = 'Standalone tests' if relative.parts[0] == 'tests' else 'Scripts/tooling' if relative.parts[0] == 'scripts' else 'Other source'
                directory, kind = ROOT/relative.parts[0], 'support'
            key = directory.relative_to(ROOT).as_posix()
            item = projects.setdefault(key, dict(project=label, kind=kind, path=key, **count(), test_example_code=0, languages={}))
            add(item, row)
            language=item['languages'].setdefault(row['language'],count())
            add(language,row)
            if any(part in ['tests', 'examples', 'benches'] for part in relative.parts) or path.stem == 'tests' or path.stem.endswith(('_tests', 'Automation', 'Checks', 'Checklist')) or path.stem == 'AutomationReports':
                item['test_example_code'] += row['code']
        else:
            category = 'documentation' if suffix in DOCS or path.name in {'LICENSE', 'NOTICE', 'AUTHORS'} else 'fixtures/data' if relative.parts[0] == 'tests' and suffix in {'.json', '.csv', '.tsv', '.sql'} else 'configuration/other text'
            add(groups[category], row)
    for key in ['blank', 'comment', 'code']:
        assert sum(row[key] for row in groups.values()) == raw['SUM'][key], key
        assert sum(row[key] for row in projects.values()) == groups['source'][key], key
    previous = []
    if base.exists():
        for path in base.glob('*/latest.json'):
            if path.parent.name != args.milestone:
                previous.append((path.stat().st_mtime, path))
    baseline = json.loads(max(previous)[1].read_text(encoding='utf-8')) if previous else None
    prior = {row['path']: row for row in baseline['projects']} if baseline else {}
    rows = sorted(projects.values(), key=lambda row: (row['kind'], row['project']))
    for row in rows:
        row['code_delta'] = row['code']-prior.get(row['path'], {}).get('code', 0) if baseline else None
    report = dict(milestone=args.milestone, recorded_utc=moment.isoformat(), method_version=1, cloc_version=version, cloc_command=command,
                  commit=git('rev-parse', 'HEAD').strip(), dirty=bool(git('status', '--porcelain').strip()), input_files=len(files), inputs_sha256=digest.hexdigest(),
                  source=groups['source'], projects=rows, languages=dict(languages), other_material={key: value for key, value in groups.items() if key != 'source'},
                  baseline=baseline['milestone'] if baseline else None, baseline_cloc_version=baseline['cloc_version'] if baseline else None,
                  notes='Physical paths counted once, including untracked nonignored source. Generated/build/vendor outputs excluded. Identical physical copies count separately; linked source belongs to its original directory. Inline tests remain in their source files; test/example column is only separately identifiable files.')
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    text = '# '+args.milestone+' code size\n\nMeasured once on the canonical checkout; cloc '+version+'.\n\n'
    text += '| Project | Code | Comments | Blank | Test/example code* | Languages (code) | Change |\n| --- | ---: | ---: | ---: | ---: | --- | ---: |\n'
    for row in rows:
        delta = '{:+,}'.format(row['code_delta']) if row['code_delta'] is not None else 'Baseline'
        languages_text=', '.join(name+' '+format(value['code'],',') for name,value in sorted(row['languages'].items()))
        text += '| '+row['project']+' | '+format(row['code'], ',')+' | '+format(row['comment'], ',')+' | '+format(row['blank'], ',')+' | '+format(row['test_example_code'], ',')+' | '+languages_text+' | '+delta+' |\n'
    source = groups['source']
    text += '\n**Total source code: '+format(source['code'], ',')+' lines** across '+str(source['files'])+' files.\n\n'
    text += '*Test/example code is a subset of Code, not an additional total. Inline tests remain in their source files.\n\n'
    text += 'Generated output, dependencies and build caches are excluded. Documentation, configuration and fixture text are recorded separately in report.json.\n'
    (output/'report.md').write_text(text, encoding='utf-8')
    (output.parent/'latest.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    (output.parent/'latest.md').write_text(text, encoding='utf-8')
    print(text)
    print('Evidence: '+str(output))


if __name__ == '__main__':
    main()
