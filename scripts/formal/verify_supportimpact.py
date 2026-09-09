"""Independent, stdlib-only consistency checks; NOT evidence authentication."""
import hashlib
import json
import re
import stat
from datetime import datetime
from pathlib import Path, PurePosixPath


def relative_name(name):
    if (not isinstance(name, str) or not name or len(name) > 240
            or ':' in name or '\\' in name or name.startswith('/')
            or any(part in ('', '.', '..') or part.endswith(('.', ' '))
                   for part in name.split('/'))):
        raise ValueError('non-portable relative path')
    return name


def safe_path(root, name):
    relative_name(name)
    path = Path(root)
    for part in ('', *PurePosixPath(name).parts):
        if part:
            path = path / part
        if path.exists() or path.is_symlink():
            info = path.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
                raise ValueError('link/reparse input is not supported')
    return path


def read_bytes(root, name):
    path = safe_path(root, name)
    if not stat.S_ISREG(path.stat().st_mode) or path.stat().st_size > 16 * 1024 * 1024:
        raise ValueError('non-regular or oversized bundle input')
    return path.read_bytes()


def read_json(root, name):
    raw = read_bytes(root, name)
    if len(raw) > 1024 * 1024:
        raise ValueError('JSON size bound exceeded')
    def unique(pairs):
        value = {}
        for key, item in pairs:
            if key in value:
                raise ValueError('duplicate JSON key')
            value[key] = item
        return value
    def invalid_constant(value):
        raise ValueError('non-finite JSON number')
    try:
        return json.loads(raw, object_pairs_hook=unique, parse_constant=invalid_constant)
    except RecursionError as error:
        raise ValueError('JSON nesting bound exceeded') from error


def reported_summary(raw):
    matches = re.findall(rb'^Dafny program verifier finished with ([0-9]+) verified, ([0-9]+) errors\r?$', raw, re.MULTILINE)
    if len(matches) != 1:
        return None
    return dict(zip(('verified', 'errors'), map(int, matches[0])))


def check_result(raw, result):
    summary = reported_summary(raw)
    if (type(result.get('actual_exit_code')) is not int
            or result['actual_exit_code'] != 0
            or result.get('timed_out') is not False
            or summary is None or summary['verified'] <= 0 or summary['errors'] != 0
            or result.get('reported_summary') != summary):
        raise ValueError('actual process result and reported summary do not establish a successful proof run')
    return summary


def digest(data):
    return hashlib.sha256(data).hexdigest()


def utc_timestamp(value):
    if not isinstance(value, str):
        raise ValueError('missing UTC timestamp')
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError as error:
        raise ValueError('invalid UTC timestamp') from error
    if parsed.tzinfo is None or parsed.utcoffset() is None:
        raise ValueError('UTC timestamp lacks offset')
    return parsed


def verify_intent(root, doc, process):
    intent = read_json(root, 'intent.json')
    expected = {
        'recorded_at_utc', 'stage', 'formal_source_sha256', 'runner_sha256',
        'verifier_sha256', 'tool_inventory_sha256', 'repository', 'invocation',
        'verification_configuration',
    }
    if set(intent) != expected or intent['stage'] != 'PRE_EXECUTION':
        raise ValueError('malformed pre-execution intent')
    recorded = utc_timestamp(intent['recorded_at_utc'])
    if (intent['formal_source_sha256'] != doc['formal_source_sha256']
            or intent['runner_sha256'] != doc['runner_sha256']
            or intent['verifier_sha256'] != digest(read_bytes(root, 'verify_supportimpact.py'))
            or intent['tool_inventory_sha256'] != digest(read_bytes(root, 'tool-inventory.json'))
            or intent['repository'] != doc['repository']
            or intent['invocation'] != doc['invocation']
            or intent['verification_configuration'] != doc['verification_configuration']):
        raise ValueError('pre-execution intent disagrees with retained execution inputs')
    for observed in process:
        if recorded > utc_timestamp(observed['started_at_utc']):
            raise ValueError('pre-execution intent postdates process start')


def verify_identities(root):
    root = Path(root)
    doc = read_json(root, 'provenance.json')
    for path_key, hash_key in (
        ('formal_source_snapshot', 'formal_source_sha256'),
        ('retained_verification_log', 'retained_verification_log_sha256'),
    ):
        expected = doc.get(hash_key)
        if not expected or digest(read_bytes(root, doc[path_key])) != expected:
            raise ValueError('missing or mismatched ' + hash_key)
    return doc


PAYLOADS = {'source/SupportImpact.dfy', 'run_supportimpact.py', 'verify_supportimpact.py',
            'tool-inventory.json', 'result.json', 'intent.json'} | {
    name + suffix for name in ('proof', 'dafny-version', 'solver-version')
    for suffix in ('.stdout.bin', '.stderr.bin', '.process.json')}


def verify(root):
    root = Path(root)
    try:
        doc = verify_identities(root)
        if doc['schema'] != 'supportimpact-future-v1':
            raise ValueError('unknown schema')
        if (doc['formal_source'] != 'formal/SupportImpact.dfy'
                or doc['formal_source_snapshot'] != 'source/SupportImpact.dfy'
                or doc['retained_verification_log'] != 'proof.stdout.bin'
                or doc['runner'] != 'run_supportimpact.py'
                or doc['log_line_endings'] != 'raw process bytes; no normalization'
                or doc['source_stable'] is not True or doc['tool_distribution_stable'] is not True):
            raise ValueError('fixed SupportImpact binding or stable observation missing')
        actual = {p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file()}
        if set(doc['payloads']) != PAYLOADS or actual != PAYLOADS | {'provenance.json'}:
            raise ValueError('incomplete or extra payload closure')
        for name, record in doc['payloads'].items():
            data = read_bytes(root, name)
            if record != {'bytes': len(data), 'sha256': digest(data)}:
                raise ValueError('payload identity mismatch: ' + name)
        if doc['runner_sha256'] != digest(read_bytes(root, 'run_supportimpact.py')):
            raise ValueError('runner identity mismatch')
        result = read_json(root, 'result.json')
        process = read_json(root, 'proof.process.json')
        if result != dict(process, reported_summary=reported_summary(read_bytes(root, 'proof.stdout.bin'))):
            raise ValueError('process/result disagreement')
        check_result(read_bytes(root, 'proof.stdout.bin'), result)
        config = doc['verification_configuration']
        timeout = config['process_timeout_seconds']
        if type(timeout) not in (int, float) or not 1 <= timeout <= 300:
            raise ValueError('process timeout out of bounds')
        if config != {'cores': 1, 'resource_limit': 100000,
                      'verification_time_limit_seconds': 30, 'process_timeout_seconds': timeout}:
            raise ValueError('configuration mismatch')
        tool, solver = doc['dafny']['executable'], doc['solver']['executable']
        if doc['invocation'] != [tool, 'verify', '--cores:1', '--resource-limit:100000',
                                 '--verification-time-limit:30', '--solver-path:' + solver,
                                 'formal/SupportImpact.dfy'] or doc['invocation_cwd'] != 'repository root':
            raise ValueError('invocation mismatch')
        for label, metadata in (('dafny-version', doc['dafny']), ('solver-version', doc['solver'])):
            observed = read_json(root, label + '.process.json')
            if (type(observed['actual_exit_code']) is not int or observed['actual_exit_code'] != 0
                    or observed['timed_out'] is not False or observed['launch_error'] is not None
                    or metadata['version'] != read_bytes(root, label + '.stdout.bin').decode('utf-8').strip()):
                raise ValueError('version observation mismatch')
        verify_intent(root, doc, (process,
                                  read_json(root, 'dafny-version.process.json'),
                                  read_json(root, 'solver-version.process.json')))
        if not doc['dafny']['version'].startswith('4.11.0') or not doc['solver']['version'].startswith('Z3 version 4.12.1 '):
            raise ValueError('unexpected tool version')
        tools = read_json(root, 'tool-inventory.json')
        if doc['dafny']['distribution_inventory'] != 'tool-inventory.json' or not tools:
            raise ValueError('missing tool inventory')
        for required in ('Dafny.exe', 'z3/bin/z3-4.12.1.exe'):
            if required not in tools:
                raise ValueError('missing executable identity')
        for value in tools.values():
            if (set(value) != {'bytes', 'sha256'} or type(value['bytes']) is not int
                    or value['bytes'] < 0 or not re.fullmatch('[0-9a-f]{64}', value['sha256'])):
                raise ValueError('malformed tool identity')
        for field in ('commit', 'tree', 'source_blob'):
            if not re.fullmatch('[0-9a-f]{40}', doc['repository'][field]):
                raise ValueError('invalid source repository identity')
        return doc
    except (KeyError, TypeError, OSError, UnicodeError) as error:
        raise ValueError('missing or malformed bundle field or payload') from error
