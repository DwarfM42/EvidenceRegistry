"""Focused FUTURE-provenance tests; temporary outputs stay in the ignored lane."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import types
import unittest

HERE = Path(__file__).resolve().parent
WORK = HERE.parents[1] / 'target/formal-provenance-tests'
WORK.mkdir(parents=True, exist_ok=True)


def load(name):
    path = HERE / (name + '.py')
    if not path.exists():
        return types.SimpleNamespace()
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def sha(data):
    return hashlib.sha256(data).hexdigest()


class IdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=WORK)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'source').mkdir()
        (self.root / 'source/SupportImpact.dfy').write_bytes(b'module Example {}\n')
        (self.root / 'stdout.bin').write_bytes(b'Dafny program verifier finished with 11 verified, 0 errors\r\n')
        self.doc = {
            'formal_source': 'formal/SupportImpact.dfy',
            'formal_source_snapshot': 'source/SupportImpact.dfy',
            'formal_source_sha256': sha((self.root / 'source/SupportImpact.dfy').read_bytes()),
            'retained_verification_log': 'stdout.bin',
            'retained_verification_log_sha256': sha((self.root / 'stdout.bin').read_bytes()),
        }

    def save(self):
        (self.root / 'provenance.json').write_text(json.dumps(self.doc), encoding='utf-8')

    def verify(self):
        self.save()
        verifier = load('verify_supportimpact')
        self.assertTrue(callable(getattr(verifier, 'verify_identities', None)), 'independent verifier missing')
        return verifier.verify_identities(self.root)

    def test_actual_exit_and_reported_summary(self):
        verifier = load('verify_supportimpact')
        self.assertTrue(callable(getattr(verifier, 'check_result', None)), 'result validation missing')
        raw = (self.root / 'stdout.bin').read_bytes()
        good = {'actual_exit_code': 0, 'timed_out': False, 'reported_summary': {'verified': 11, 'errors': 0}}
        verifier.check_result(raw, good)
        negatives = [dict(good, actual_exit_code=1), dict(good, timed_out=True),
                     dict(good, reported_summary={'verified': 10, 'errors': 0}),
                     dict(good, actual_exit_code=False)]
        for result in negatives:
            with self.assertRaises(ValueError):
                verifier.check_result(raw, result)
        for output in (b'', raw + raw, raw.replace(b'0 errors', b'1 errors'),
                       raw.replace(b'11 verified', b'0 verified')):
            with self.assertRaises(ValueError):
                verifier.check_result(output, good)

    def test_strict_json_and_portable_file_boundary(self):
        verifier = load('verify_supportimpact')
        self.save()
        raw = (self.root / 'provenance.json').read_text()
        (self.root / 'provenance.json').write_text(raw[:-1] + ', "formal_source_sha256": "' + self.doc['formal_source_sha256'] + '"}')
        with self.assertRaises(ValueError, msg='duplicate JSON must be rejected'):
            verifier.verify_identities(self.root)
        original = self.doc['formal_source_snapshot']
        for name in ('source/../source/SupportImpact.dfy', str(self.root / original),
                     'source\\SupportImpact.dfy', './source/SupportImpact.dfy'):
            self.doc['formal_source_snapshot'] = name
            with self.assertRaises(ValueError, msg='unsafe path ' + name):
                self.verify()
        self.doc['formal_source_snapshot'] = original
        (self.root / original).write_bytes(b'changed source')
        with self.assertRaises(ValueError):
            self.verify()

    def test_link_input_is_rejected(self):
        link = self.root / 'linked.dfy'
        try:
            link.symlink_to(self.root / self.doc['formal_source_snapshot'])
        except OSError:
            self.skipTest('symlink creation is not permitted on this host')
        self.doc['formal_source_snapshot'] = 'linked.dfy'
        with self.assertRaises(ValueError):
            self.verify()

    def test_source_and_log_identity(self):
        self.verify()
        for field in ('formal_source_sha256', 'retained_verification_log_sha256'):
            original = self.doc[field]
            del self.doc[field]
            with self.assertRaises(ValueError, msg='missing ' + field):
                self.verify()
            self.doc[field] = '0' * 64
            with self.assertRaises(ValueError, msg='mismatched ' + field):
                self.verify()
            self.doc[field] = original


import sys
import subprocess
from unittest import mock


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=WORK)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_future_run_binds_source_tool_runner_and_result(self):
        runner = load('run_supportimpact')
        self.assertTrue(callable(getattr(runner, 'run', None)), 'SupportImpact run assembly missing')
        repo = HERE.parents[1]
        toolroot = self.root / 'tool'
        (toolroot / 'z3/bin').mkdir(parents=True)
        (toolroot / 'Dafny.exe').write_bytes(b'SYNTHETIC TEST TOOL; NOT EXECUTED')
        (toolroot / 'z3/bin/z3-4.12.1.exe').write_bytes(b'SYNTHETIC TEST SOLVER; NOT EXECUTED')
        tool = (toolroot / 'Dafny.exe').relative_to(repo).as_posix()
        out = (self.root / 'bundle').relative_to(repo).as_posix()

        # Only process launching is substituted here. Real subprocess behavior is
        # exercised separately; these synthetic bytes are never public proof evidence.
        def synthetic_capture(argv, cwd, destination, name, timeout):
            self.assertTrue((destination / 'intent.json').exists(), 'pre-execution intent missing')
            intent = json.loads((destination / 'intent.json').read_text())
            self.assertEqual(intent['formal_source_sha256'], sha((repo / 'formal/SupportImpact.dfy').read_bytes()))
            self.assertEqual(intent['runner_sha256'], sha((HERE / 'run_supportimpact.py').read_bytes()))
            self.assertEqual(intent['invocation'][0], tool)
            raw = {'dafny-version': b'4.11.0+test\r\n',
                   'solver-version': b'Z3 version 4.12.1 - 64 bit\r\n',
                   'proof': b'Dafny program verifier finished with 11 verified, 0 errors\r\n'}[name]
            (destination / (name + '.stdout.bin')).write_bytes(raw)
            (destination / (name + '.stderr.bin')).write_bytes(b'')
            result = {'actual_exit_code': 0, 'timed_out': False, 'launch_error': None,
                      'started_at_utc': runner.utc_now(), 'ended_at_utc': runner.utc_now()}
            runner.write_json(destination / (name + '.process.json'), result)
            return result

        with mock.patch.object(runner, 'capture', synthetic_capture):
            document = runner.run(repo, out, tool, timeout=30)
            self.assertEqual(document['formal_source_sha256'], sha((repo / 'formal/SupportImpact.dfy').read_bytes()))
            self.assertEqual(document['retained_verification_log_sha256'], sha((repo / out / 'proof.stdout.bin').read_bytes()))
            self.assertEqual(document['repository']['commit'], subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo).decode().strip())
            self.assertEqual(document['runner_sha256'], sha((HERE / 'run_supportimpact.py').read_bytes()))
            self.assertEqual(document['dafny']['version'], '4.11.0+test')
            self.assertIn('--cores:1', document['invocation'])
            self.assertIn('--resource-limit:100000', document['invocation'])
            load('verify_supportimpact').verify(repo / out)
            before = {p.relative_to(repo / out).as_posix(): p.read_bytes() for p in (repo / out).rglob('*') if p.is_file()}
            with self.assertRaises(FileExistsError):
                runner.run(repo, out, tool, timeout=30)
            self.assertEqual(before, {p.relative_to(repo / out).as_posix(): p.read_bytes() for p in (repo / out).rglob('*') if p.is_file()})
            verifier = load('verify_supportimpact')
            bundle = repo / out
            for name in ('run_supportimpact.py', 'verify_supportimpact.py', 'result.json',
                         'proof.process.json', 'proof.stderr.bin', 'tool-inventory.json'):
                original = (bundle / name).read_bytes()
                (bundle / name).write_bytes(original + b' ')
                with self.assertRaises(ValueError, msg='unbound payload: ' + name):
                    verifier.verify(bundle)
                (bundle / name).write_bytes(original)
            original_doc = (bundle / 'provenance.json').read_bytes()
            for mutate in (
                lambda d: d['payloads'].pop('result.json'),
                lambda d: d.update(source_stable=False),
                lambda d: d.update(tool_distribution_stable=False),
                lambda d: d['verification_configuration'].update(cores=2),
                lambda d: d['dafny'].update(version='invented'),
                lambda d: d.update(invocation=['invented']),
            ):
                changed = copy.deepcopy(document)
                mutate(changed)
                (bundle / 'provenance.json').write_text(json.dumps(changed))
                with self.assertRaises(ValueError):
                    verifier.verify(bundle)
                (bundle / 'provenance.json').write_bytes(original_doc)
            for field in ('invocation', 'formal_source_sha256', 'runner_sha256', 'verifier_sha256',
                          'tool_inventory_sha256', 'recorded_at_utc'):
                original = (bundle / 'intent.json').read_bytes()
                intent = json.loads(original)
                intent.pop(field)
                (bundle / 'intent.json').write_text(json.dumps(intent))
                changed = copy.deepcopy(document)
                changed['payloads']['intent.json'] = runner.identity(bundle / 'intent.json')
                (bundle / 'provenance.json').write_text(json.dumps(changed))
                with self.assertRaises(ValueError, msg='missing pre-execution binding ' + field):
                    verifier.verify(bundle)
                (bundle / 'intent.json').write_bytes(original)
                (bundle / 'provenance.json').write_bytes(original_doc)
            result = json.loads((bundle / 'result.json').read_text())
            result['actual_exit_code'] = 1
            (bundle / 'result.json').write_text(json.dumps(result))
            changed = copy.deepcopy(document)
            changed['payloads']['result.json'] = runner.identity(bundle / 'result.json')
            (bundle / 'provenance.json').write_text(json.dumps(changed))
            with self.assertRaises(ValueError, msg='self-consistent digest cannot hide actual exit conflict'):
                verifier.verify(bundle)

    def test_timeout_and_launch_failure_are_retained(self):
        runner = load('run_supportimpact')
        out = self.root / 'timeout'
        out.mkdir()
        try:
            result = runner.capture([sys.executable, '-c', "import time; print('started', flush=True); time.sleep(10)"], self.root, out, 'proof', 0.5)
        except subprocess.TimeoutExpired:
            self.fail('timeout must return retained process evidence, not escape')
        self.assertIs(result['timed_out'], True)
        self.assertIn(b'started', (out / 'proof.stdout.bin').read_bytes())
        self.assertEqual(json.loads((out / 'proof.process.json').read_text()), result)
        failed = self.root / 'launch'
        failed.mkdir()
        result = runner.capture([str(self.root / 'absent-executable')], self.root, failed, 'proof', 1)
        self.assertIsNone(result['actual_exit_code'])
        self.assertEqual(result['launch_error'], 'FileNotFoundError')
        self.assertTrue((failed / 'proof.stderr.bin').exists())

    def test_failed_proof_retains_raw_result_without_provenance(self):
        runner = load('run_supportimpact')
        repo = HERE.parents[1]
        toolroot = self.root / 'failed-tool'
        (toolroot / 'z3/bin').mkdir(parents=True)
        (toolroot / 'Dafny.exe').write_bytes(b'SYNTHETIC TEST TOOL; NOT EXECUTED')
        (toolroot / 'z3/bin/z3-4.12.1.exe').write_bytes(b'SYNTHETIC TEST SOLVER; NOT EXECUTED')
        tool = (toolroot / 'Dafny.exe').relative_to(repo).as_posix()
        out = (self.root / 'failed-bundle').relative_to(repo).as_posix()

        def failed_capture(argv, cwd, destination, name, timeout):
            raw = {'dafny-version': b'4.11.0+test\r\n',
                   'solver-version': b'Z3 version 4.12.1 - 64 bit\r\n',
                   'proof': b'Dafny program verifier finished with 0 verified, 1 errors\r\n'}[name]
            (destination / (name + '.stdout.bin')).write_bytes(raw)
            (destination / (name + '.stderr.bin')).write_bytes(b'proof failed' if name == 'proof' else b'')
            result = {'actual_exit_code': 1 if name == 'proof' else 0, 'timed_out': False,
                      'launch_error': None, 'started_at_utc': runner.utc_now(),
                      'ended_at_utc': runner.utc_now()}
            runner.write_json(destination / (name + '.process.json'), result)
            return result

        with mock.patch.object(runner, 'capture', failed_capture):
            with self.assertRaises(ValueError):
                runner.run(repo, out, tool, timeout=30)
        bundle = repo / out
        self.assertTrue((bundle / 'result.json').is_file())
        self.assertTrue((bundle / 'proof.stdout.bin').is_file())
        self.assertFalse((bundle / 'provenance.json').exists())

    def test_raw_process_capture_and_exclusive_outputs(self):
        runner = load('run_supportimpact')
        self.assertTrue(callable(getattr(runner, 'capture', None)), 'bounded capture runner missing')
        out = self.root / 'run'
        out.mkdir()
        command = [sys.executable, '-c', "import sys; sys.stdout.buffer.write(b'raw\\r\\n'); sys.stderr.buffer.write(b'err\\x00'); sys.exit(7)"]
        result = runner.capture(command, self.root, out, 'proof', 10)
        self.assertEqual(result['actual_exit_code'], 7)
        self.assertIs(result['timed_out'], False)
        self.assertIn('started_at_utc', result)
        self.assertIn('ended_at_utc', result)
        self.assertLessEqual(result['started_at_utc'], result['ended_at_utc'])
        self.assertEqual((out / 'proof.stdout.bin').read_bytes(), b'raw\r\n')
        self.assertEqual((out / 'proof.stderr.bin').read_bytes(), b'err\x00')
        before = {p.name: p.read_bytes() for p in out.iterdir()}
        with self.assertRaises(FileExistsError):
            runner.capture(command, self.root, out, 'proof', 10)
        self.assertEqual(before, {p.name: p.read_bytes() for p in out.iterdir()})


if __name__ == '__main__':
    unittest.main()
