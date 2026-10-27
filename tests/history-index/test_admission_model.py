"""Adversarial examples validate proposed arithmetic, not backend enforcement."""
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from research_history_admission_model import Proposal,Refusal,plan,bounds


def key(value):return value.to_bytes(32,'big')


class AdmissionModelTests(unittest.TestCase):
    def test_eight_split_chain_hits_tight_bounds(self):
        # Existing depth 16, seven shared zero bits, then a separating bit at depth 24.
        keys=[key(n) for n in range(30)]+[key((1<<232)+n) for n in range(31)]
        result=plan(1<<16,keys[:-1],keys[-1],100)
        self.assertEqual((result['splits'],result['node_writes'],result['logical_node_bytes']), (8,33,5473))
        self.assertEqual(result['packed_node_dml_statements'],57)
        self.assertEqual(sum(n for _,n in result['leaf_counts']),61)

    def test_depth_refusal_even_with_free_leaves(self):
        with self.assertRaisesRegex(Refusal,'depth'):plan(1<<24,[key(n) for n in range(60)],key(60),100)

    def test_split_refusal_before_depth_limit(self):
        with self.assertRaisesRegex(Refusal,'split'):plan(1,[key(n) for n in range(60)],key(60),1)

    def test_leaf_cap_still_allows_non_splitting_insert(self):
        result=plan(1<<12,[key(0)],key(1),4096)
        self.assertEqual((result['splits'],result['new_leaves']),(0,4096))

    def test_leaf_cap_refuses_splitting_insert(self):
        keys=[key(n) for n in range(30)]+[key((1<<243)+n) for n in range(31)]
        with self.assertRaisesRegex(Refusal,'leaf'):plan(1<<12,keys[:-1],keys[-1],4096)

    def test_final_available_leaf_is_admitted(self):
        keys=[key(n) for n in range(30)]+[key((1<<243)+n) for n in range(31)]
        result=plan(1<<12,keys[:-1],keys[-1],4095)
        self.assertEqual(result['new_leaves'],4096)

    def test_duplicate_and_wrong_interval_are_refusals(self):
        with self.assertRaises(Refusal):plan(1,[key(1)],key(1),1)
        with self.assertRaises(Refusal):plan(2,[],key(1<<255),1)

    def test_bad_configuration_and_shape(self):
        for kwargs in [{'leaves':4097},{'entries':61},{'splits':9},{'depth':25}]:
            with self.assertRaises(Refusal):Proposal(**kwargs)
        for bucket,keys,new,leaves in [(0,[],key(0),1),(1,[],b'x',1),(1,[],key(0),0)]:
            with self.assertRaises(Refusal):plan(bucket,keys,new,leaves)

    def test_budget_bounds_match_constructed_extreme(self):
        result=bounds();self.assertEqual(result['structural_entry_ceiling'],245760)
        self.assertEqual(result['current_node_ceiling'],8191)
        self.assertEqual(result['proof_node_bytes'],5401)
        self.assertEqual(result['publication_node_writes'],33)
        self.assertEqual(result['publication_index_bytes_with_64B_envelope'],5537)

    def test_proposal_does_not_enable_new_backend_configuration(self):
        import tempfile
        from research_history_hierarchy import PackedStore
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaisesRegex(ValueError,'resource contract'):
                PackedStore(Path(temp)/'not-created.sqlite',bytes(32),'buckets',max_leaves=4096)
            self.assertFalse((Path(temp)/'not-created.sqlite').exists())


if __name__=='__main__':unittest.main()
