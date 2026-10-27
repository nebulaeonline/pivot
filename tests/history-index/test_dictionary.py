"""Bounded native zstd blocks and explicit dictionary dependence; no fallback."""
import os
from pathlib import Path
import sys
import unittest

ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from research_history_dictionary import Zstd
from research_history_compact import MAX_BYTES,varint,sha
from research_history_index import Unknown


@unittest.skipUnless(os.environ.get('PIVOT_ZSTD_LIBRARY'),'explicit native research library required')
class DictionaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.library=Path(os.environ['PIVOT_ZSTD_LIBRARY']);z=Zstd(cls.library)
        cls.samples=[b'message-kind:edit;content:'+sha(str(i).encode())+b';fields:'+b'amount,formula,identity;'*9 for i in range(400)]
        cls.raw=b'message-kind:edit;content:'+sha(b'unseen')+b';fields:'+b'amount,formula,identity;'*9
        cls.dictionary=z.train(cls.samples,512);z.close()
    def setUp(self):self.z=Zstd(self.library)
    def tearDown(self):self.z.close()
    def test_plain_and_trained_roundtrip(self):
        ident=self.z.load(self.dictionary,3)
        for selected in [0,ident]:
            block=self.z.encode(self.raw,3,selected)
            self.assertEqual(self.z.decode(block),self.raw)
            self.assertEqual(block[0],2 if selected else 1)
    def test_missing_dictionary_is_refusal_without_fallback(self):
        block=self.z.encode(self.raw,3,self.z.load(self.dictionary,3));empty=Zstd(self.library)
        try:
            with self.assertRaises(Unknown):empty.decode(block)
        finally:empty.close()
    def test_explicit_raw_representation(self):
        raw=sha(b'first')+sha(b'second');block=self.z.encode(raw,3)
        self.assertEqual(block[0],0);self.assertEqual(self.z.decode(block),raw)
    def test_truncation_and_trailing_data_refuse(self):
        block=self.z.encode(self.raw,3)
        for invalid in [b'',block[:-3],block+b'garbage',b'\x07'+block[1:],b'\x00'+varint(10)+b'x']:
            with self.assertRaises(Unknown):self.z.decode(invalid)
    def test_allocation_limit_and_canonical_lengths(self):
        for invalid in [b'\x01'+varint(MAX_BYTES+1),b'\x00\x81\x00x',b'\x01\x80'*8]:
            with self.assertRaises(Unknown):self.z.decode(invalid)
    def test_training_admission_and_memory_accounting(self):
        with self.assertRaises(ValueError):self.z.train(self.samples[:10],512)
        self.z.load(self.dictionary,3);memory=self.z.memory()
        self.assertEqual(memory['serialized_dictionary_bytes'],len(self.dictionary))
        self.assertGreater(memory['dictionary_load_ns'],0)


if __name__=='__main__':unittest.main()
