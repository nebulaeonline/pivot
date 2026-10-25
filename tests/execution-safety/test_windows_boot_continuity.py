"""Production comparator and wrapper integration only; no OS GUI or clipboard."""
from copy import deepcopy
from datetime import datetime, timedelta
import math
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts/qualification'))
import windows_boot_continuity as policy
import windows_attended_gui as wrapper
from test_windows_attended_gui import reviewed_fixture


class BootContinuityTests(unittest.TestCase):
    def current(self):
        sample = deepcopy(policy.REVIEWED_SAMPLE)
        for point in ['before', 'after']:
            sample[point]['utc_unix'] += 3600
            sample[point]['uptime_ms'] += 3600000
        return sample

    def compare(self, sample):
        return policy.compare(policy.REVIEWED_SAMPLE, sample)

    def test_stable_long_interval_and_separate_query_diagnostics(self):
        result = self.compare(self.current())
        self.assertEqual(result['status'], 'Verified')
        self.assertEqual([p['kind'] for p in result['intervals']],
                         ['baseline-query', 'current-query', 'baseline-to-current'])
        self.assertEqual(result['intervals'][2]['wall_seconds'], 3600)
        self.assertTrue(all(p['discrepancy_seconds'] <= 2 for p in result['intervals']))

    def test_observed_fractional_mismatch_is_allowed(self):
        sample = self.current()
        sample['measurement']['last_boot_utc'] = '2026-10-25T04:37:42.0920420Z'
        result = self.compare(sample)
        self.assertEqual(result['status'], 'Verified')
        self.assertAlmostEqual(result['CIM_variation_seconds'], .001179)

    def test_explicit_cim_budget_boundaries_and_submillisecond_values(self):
        for delta, expected in [(.000001, 'Verified'), (.099999, 'Verified'), (.1, 'Verified'),
                                (.100001, 'Blocked'), (60, 'Blocked')]:
            sample = self.current()
            sample['measurement']['last_boot_utc'] = (datetime.fromisoformat(policy.REVIEWED_SAMPLE['measurement']['last_boot_utc'])+timedelta(seconds=delta)).isoformat()
            result = self.compare(sample)
            self.assertEqual(result['status'], expected)
            if expected == 'Blocked': self.assertEqual(result['classification'], 'continuity-uncertain')

    def test_all_anchor_fields_are_required_and_exact(self):
        for field, value in [('provider', 'Other'), ('channel', 'Other'), ('event_id', 13),
                             ('record_id', 2911), ('time_utc', '2026-10-25T04:37:39Z'),
                             ('start_time', '2026-10-25T04:37:39Z')]:
            sample = self.current(); sample['measurement']['start_event'][field] = value
            self.assertEqual(self.compare(sample)['status'], 'Blocked')
        sample = self.current(); sample['measurement']['start_event']['record_id'] = 2911
        self.assertEqual(self.compare(sample)['classification'], 'anchor-mismatch')

    def test_stale_event_with_reset_or_clock_step_is_uncertain(self):
        for field, value in [('uptime_ms', 10), ('utc_unix', 0)]:
            sample = self.current(); sample['before'][field] = value
            self.assertEqual(self.compare(sample)['status'], 'Blocked')

    def test_query_budget_and_cross_observation_budget_independently(self):
        for delta, expected in [(2, 'Verified'), (2.001, 'Blocked')]:
            sample = self.current(); sample['after']['utc_unix'] = sample['before']['utc_unix']+1+delta
            sample['after']['uptime_ms'] = sample['before']['uptime_ms']+1000
            self.assertEqual(self.compare(sample)['status'], expected)
            sample = self.current()
            for point in ['before', 'after']: sample[point]['utc_unix'] += delta
            self.assertEqual(self.compare(sample)['status'], expected)

    def test_missing_malformed_or_unavailable_measurements(self):
        for value in [None, {}, [], {'query_exit_code': 1}]: self.assertEqual(self.compare(value)['status'], 'Blocked')
        for section, field, value in [('measurement', 'start_event', None),
                                      ('measurement', 'last_boot_utc', '2026-10-25T04:37:42'),
                                      ('before', 'uptime_ms', True), ('after', 'utc_unix', math.nan),
                                      ('before', 'uptime_ms', -1)]:
            sample = self.current(); sample[section][field] = value
            self.assertEqual(self.compare(sample)['status'], 'Blocked')
        for key in ['provider', 'channel', 'event_id', 'record_id', 'time_utc', 'start_time']:
            sample = self.current(); del sample['measurement']['start_event'][key]
            self.assertEqual(self.compare(sample)['status'], 'Blocked')

    def test_wrapper_accepts_timestamp_observation_variation_not_context_change(self):
        before, after = reviewed_fixture(), reviewed_fixture()
        after['boot'] = 'presentation can differ; raw policy measurement is separate'
        after['boot_observation'] = self.current()
        self.assertEqual(wrapper.identity_continuity(before, after)['status'], 'Verified')
        for field, value in [('user', 'Other'), ('session_id', 2), ('input_desktop', 'Winlogon')]:
            changed = deepcopy(after); changed['desktop'][field] = value
            self.assertEqual(wrapper.identity_continuity(before, changed)['status'], 'Blocked')
            self.assertEqual(wrapper.reviewed_identity(changed)['status'], 'Blocked')
        for identity in [None, {}, {'desktop': None}]:
            self.assertEqual(wrapper.reviewed_identity(identity)['status'], 'Blocked')
        changed = reviewed_fixture(); changed['uuid'] = 'different VM'
        self.assertEqual(wrapper.reviewed_identity(changed)['status'], 'Blocked')


if __name__ == '__main__': unittest.main()
