"""Bounded future SupportImpact capture. Existing evidence is never overwritten."""
import json
import os
import signal
from pathlib import Path
import subprocess
import platform
from datetime import datetime, timezone
from verify_supportimpact import check_result, digest, reported_summary, verify


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def write_json(path, value):
    with Path(path).open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def capture(argv, cwd, out, name, timeout):
    out = Path(out)
    result = {'actual_exit_code': None, 'timed_out': False, 'launch_error': None,
              'started_at_utc': utc_now()}
    with (out / (name + '.stdout.bin')).open('xb') as stdout:
        with (out / (name + '.stderr.bin')).open('xb') as stderr:
            try:
                with subprocess.Popen(argv, cwd=cwd, stdout=stdout, stderr=stderr,
                                      start_new_session=(os.name != 'nt')) as process:
                    try:
                        process.wait(timeout=timeout)
                    except subprocess.TimeoutExpired:
                        result['timed_out'] = True
                        if os.name == 'nt':
                            # Terminate this invocation's tree, never unrelated processes.
                            subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                           timeout=10, check=False)
                        else:
                            os.killpg(process.pid, signal.SIGKILL)
                        if process.poll() is None:
                            process.kill()
                        process.wait(timeout=10)
                    result['actual_exit_code'] = process.returncode
            except OSError as error:
                result['launch_error'] = type(error).__name__
    result['ended_at_utc'] = utc_now()
    write_json(out / (name + '.process.json'), result)
    return result


def identity(path):
    data = Path(path).read_bytes()
    return {'bytes': len(data), 'sha256': digest(data)}


def inventory(root):
    return {p.relative_to(root).as_posix(): identity(p)
            for p in sorted(Path(root).rglob('*')) if p.is_file()}


def git(repo, *args):
    return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=repo, timeout=15)


def run(repo, output, tool, timeout=120):
    repo = Path(repo).resolve()
    out = repo / output
    out.mkdir(exist_ok=False)
    source_name = 'formal/SupportImpact.dfy'
    source = (repo / source_name).read_bytes()
    commit = git(repo, 'rev-parse', 'HEAD').decode().strip()
    tree = git(repo, 'rev-parse', commit + '^{tree}').decode().strip()
    if source != git(repo, 'show', commit + ':' + source_name):
        raise ValueError('worktree formal source differs from source commit')
    (out / 'source').mkdir()
    (out / 'source/SupportImpact.dfy').write_bytes(source)
    for name in ('run_supportimpact.py', 'verify_supportimpact.py'):
        (out / name).write_bytes(Path(__file__).with_name(name).read_bytes())
    toolroot = (repo / tool).parent
    tools_before = inventory(toolroot)
    write_json(out / 'tool-inventory.json', tools_before)
    solver = (Path(tool).parent / 'z3/bin/z3-4.12.1.exe').as_posix()
    argv = [tool, 'verify', '--cores:1', '--resource-limit:100000',
            '--verification-time-limit:30', '--solver-path:' + solver, source_name]
    config = {'cores': 1, 'resource_limit': 100000, 'verification_time_limit_seconds': 30,
              'process_timeout_seconds': timeout}
    repository = {'url': 'https://github.com/DwarfM42/EvidenceRegistry', 'commit': commit,
                  'tree': tree, 'source_blob': git(repo, 'rev-parse', commit + ':' + source_name).decode().strip()}
    intent = {'recorded_at_utc': utc_now(), 'stage': 'PRE_EXECUTION',
              'formal_source_sha256': digest(source),
              'runner_sha256': digest((out / 'run_supportimpact.py').read_bytes()),
              'verifier_sha256': digest((out / 'verify_supportimpact.py').read_bytes()),
              'tool_inventory_sha256': digest((out / 'tool-inventory.json').read_bytes()),
              'repository': repository, 'invocation': argv, 'verification_configuration': config}
    write_json(out / 'intent.json', intent)
    versions = {}
    for name, version_argv in (('dafny-version', [tool, '--version']),
                       ('solver-version', [solver, '-version'])):
        observed = capture(version_argv, repo, out, name, min(timeout, 30))
        if observed['actual_exit_code'] != 0 or observed['timed_out']:
            raise ValueError('version probe failed; raw attempt retained')
        versions[name] = (out / (name + '.stdout.bin')).read_bytes().decode('utf-8').strip()
    result = capture(argv, repo, out, 'proof', timeout)
    result['reported_summary'] = reported_summary((out / 'proof.stdout.bin').read_bytes())
    write_json(out / 'result.json', result)
    # Keep raw failed/timeout attempts, but never mint provenance for them.
    check_result((out / 'proof.stdout.bin').read_bytes(), result)
    doc = {
        'schema': 'supportimpact-future-v1',
        'status': 'external FUTURE provenance; not an EvidenceRegistry Formal Verification Record',
        'claim_scope': 'PURE_FORMAL model only; no runtime correctness, authority, admission, qualification or release claim',
        'repository': repository,
        'formal_source': source_name,
        'formal_source_snapshot': 'source/SupportImpact.dfy',
        'formal_source_sha256': digest(source),
        'retained_verification_log': 'proof.stdout.bin',
        'retained_verification_log_sha256': digest((out / 'proof.stdout.bin').read_bytes()),
        'log_line_endings': 'raw process bytes; no normalization',
        'runner': 'run_supportimpact.py',
        'runner_sha256': digest((out / 'run_supportimpact.py').read_bytes()),
        'invocation': argv,
        'invocation_cwd': 'repository root',
        'verification_configuration': config,
        'dafny': {'executable': tool, 'version': versions['dafny-version'],
                  'distribution_inventory': 'tool-inventory.json'},
        'solver': {'executable': solver, 'version': versions['solver-version']},
        'runtime': {'os': platform.system(), 'machine': platform.machine(),
                    'python_version': platform.python_version()},
        'tool_scope': 'separately retained distribution inventory, not loaded-module tracing; inherited environment and OS dependencies are not closed or hermetic',
        'source_stable': source == (repo / source_name).read_bytes(),
        'tool_distribution_stable': tools_before == inventory(toolroot),
        'payloads': inventory(out),
    }
    write_json(out / 'provenance.json', doc)
    verify(out)
    return doc
