"""Offline, lossless zstd dictionary investigation; not a production codec choice."""
import argparse
import ctypes as C
import hashlib
import json
from pathlib import Path
import shutil
import sqlite3
import sys
import time
import zlib

from research_history_compact import MAX_BYTES,sha,frame,unframe,varint,decompress
from research_history_hierarchy import PackedStore
from research_history_hierarchy_measure import allocation,logical_manifest
from research_history_layout_measure import pages
from research_history_index import Unknown,distribution,checkpoint_bytes

CATALOG_SCHEMA='CREATE TABLE codec(singleton INTEGER PRIMARY KEY CHECK(singleton=1),body BLOB NOT NULL,hash BLOB NOT NULL);'


class Zstd:
    def __init__(self,path):
        self.lib=C.CDLL(str(path));self.dictionary={};self.loaded={}
        specs={
            'ZSTD_versionString':(C.c_char_p,[]),'ZSTD_isError':(C.c_uint,[C.c_size_t]),
            'ZSTD_getErrorName':(C.c_char_p,[C.c_size_t]),'ZSTD_compressBound':(C.c_size_t,[C.c_size_t]),
            'ZSTD_createCCtx':(C.c_void_p,[]),'ZSTD_createDCtx':(C.c_void_p,[]),
            'ZSTD_freeCCtx':(C.c_size_t,[C.c_void_p]),'ZSTD_freeDCtx':(C.c_size_t,[C.c_void_p]),
            'ZSTD_createCDict':(C.c_void_p,[C.c_void_p,C.c_size_t,C.c_int]),
            'ZSTD_createDDict':(C.c_void_p,[C.c_void_p,C.c_size_t]),
            'ZSTD_freeCDict':(C.c_size_t,[C.c_void_p]),'ZSTD_freeDDict':(C.c_size_t,[C.c_void_p]),
            'ZSTD_compressCCtx':(C.c_size_t,[C.c_void_p,C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t,C.c_int]),
            'ZSTD_decompressDCtx':(C.c_size_t,[C.c_void_p,C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t]),
            'ZSTD_compress_usingCDict':(C.c_size_t,[C.c_void_p,C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t,C.c_void_p]),
            'ZSTD_decompress_usingDDict':(C.c_size_t,[C.c_void_p,C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t,C.c_void_p]),
            'ZSTD_findFrameCompressedSize':(C.c_size_t,[C.c_void_p,C.c_size_t]),
            'ZSTD_getDictID_fromDict':(C.c_uint,[C.c_void_p,C.c_size_t]),
            'ZSTD_getDictID_fromFrame':(C.c_uint,[C.c_void_p,C.c_size_t]),
            'ZDICT_trainFromBuffer':(C.c_size_t,[C.c_void_p,C.c_size_t,C.c_void_p,C.POINTER(C.c_size_t),C.c_uint]),
        }
        for name,(restype,args) in specs.items():
            fn=getattr(self.lib,name);fn.restype=restype;fn.argtypes=args
        self.cc=self.lib.ZSTD_createCCtx();self.dc=self.lib.ZSTD_createDCtx()
        if not self.cc or not self.dc:raise MemoryError('zstd contexts')

    def check(self,value):
        if self.lib.ZSTD_isError(value):raise Unknown(self.lib.ZSTD_getErrorName(value).decode())
        return value

    def train(self,samples,capacity):
        if len(samples)<64 or sum(map(len,samples))<capacity*100:raise ValueError('insufficient independent training samples/bytes')
        raw=b''.join(samples);sizes=(C.c_size_t*len(samples))(*map(len,samples));out=C.create_string_buffer(capacity)
        length=self.check(self.lib.ZDICT_trainFromBuffer(out,capacity,raw,sizes,len(samples)))
        return out.raw[:length]

    def load(self,raw,level):
        ident=self.lib.ZSTD_getDictID_fromDict(raw,len(raw))
        if not ident:raise Unknown('trained dictionary has no ID')
        if ident in self.dictionary:
            if self.dictionary[ident]!=raw:raise Unknown('dictionary ID collision')
            return ident
        start=time.perf_counter_ns();cd=self.lib.ZSTD_createCDict(raw,len(raw),level);dd=self.lib.ZSTD_createDDict(raw,len(raw))
        if not cd or not dd:raise Unknown('dictionary load refused')
        self.dictionary[ident]=raw;self.loaded[ident]=(cd,dd,time.perf_counter_ns()-start)
        return ident

    def memory(self):
        result={'serialized_dictionary_bytes':sum(map(len,self.dictionary.values())),'dictionary_load_ns':sum(v[2] for v in self.loaded.values())}
        for name,pointers in [('CCtx',[self.cc]),('DCtx',[self.dc]),('CDict',[v[0] for v in self.loaded.values()]),('DDict',[v[1] for v in self.loaded.values()])]:
            fn=getattr(self.lib,'ZSTD_sizeof_'+name,None)
            if fn:
                fn.restype=C.c_size_t;fn.argtypes=[C.c_void_p];result[name+'_native_bytes']=sum(fn(ptr) for ptr in pointers)
        return result

    def encode(self,raw,level,ident=0):
        if len(raw)>MAX_BYTES:raise Unknown('raw resource bound')
        out=C.create_string_buffer(self.lib.ZSTD_compressBound(len(raw)))
        if ident:length=self.lib.ZSTD_compress_usingCDict(self.cc,out,len(out),raw,len(raw),self.loaded[ident][0])
        else:length=self.lib.ZSTD_compressCCtx(self.cc,out,len(out),raw,len(raw),level)
        compressed=out.raw[:self.check(length)]
        # Raw is an explicit stored encoding, never a decoder fallback.
        flag=2 if ident else 1
        if len(compressed)>=len(raw):flag=0;compressed=raw
        return bytes([flag])+varint(len(raw))+compressed

    def decode(self,block):
        if not block:raise Unknown('missing codec tag')
        flag=block[0];at=1;length=0
        for shift in range(0,35,7):
            if at>=len(block):raise Unknown('truncated length')
            byte=block[at];at+=1;length|=(byte&127)<<shift
            if byte<128:break
        else:raise Unknown('oversized length')
        if length>MAX_BYTES or block[1:at]!=varint(length):raise Unknown('decoded resource/noncanonical length')
        raw=block[at:]
        if flag==0:
            if len(raw)!=length:raise Unknown('raw length mismatch')
            return raw
        if flag not in (1,2) or not raw:raise Unknown('unsupported codec tag')
        if self.check(self.lib.ZSTD_findFrameCompressedSize(raw,len(raw)))!=len(raw):raise Unknown('trailing/concatenated frame')
        ident=self.lib.ZSTD_getDictID_fromFrame(raw,len(raw))
        if (flag==1 and ident) or (flag==2 and ident not in self.loaded):raise Unknown('missing/wrong dictionary; no fallback')
        out=C.create_string_buffer(length)
        if flag==2:written=self.lib.ZSTD_decompress_usingDDict(self.dc,out,length,raw,len(raw),self.loaded[ident][1])
        else:written=self.lib.ZSTD_decompressDCtx(self.dc,out,length,raw,len(raw))
        if self.check(written)!=length:raise Unknown('decoded length mismatch')
        return out.raw[:length]

    def close(self):
        for cd,dd,_ in self.loaded.values():self.lib.ZSTD_freeCDict(cd);self.lib.ZSTD_freeDDict(dd)
        self.loaded.clear();self.dictionary.clear()
        self.lib.ZSTD_freeCCtx(self.cc);self.lib.ZSTD_freeDCtx(self.dc)


class CodecStore(PackedStore):
    def __init__(self,path,instance,mode,*,codec,**kwargs):
        self.codec=codec
        super().__init__(path,instance,mode,**kwargs)
        self.schema_sql+=CATALOG_SCHEMA;model=sqlite3.connect(':memory:')
        try:
            model.executescript(self.schema_sql);self.schema=set(model.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema'))
        finally:model.close()
        row=self.db.execute('SELECT body,hash FROM codec WHERE singleton=1').fetchone()
        if row is None or sha(row[0])!=row[1]:self.bad('codec catalog checksum')
        self.format_id=b'G3Z1'+row[1]
        fields=unframe(row[0])
        if fields[:1]!=[b'Z1'] or (len(fields)-1)%3:self.bad('codec catalog format')
        for i in range(1,len(fields),3):
            ident,digest,length=fields[i:i+3]
            raw=self.codec.dictionary.get(ident)
            if raw is None or len(raw)!=length or sha(raw)!=digest:self.bad('dictionary catalog binding')

    def canonical(self,row):
        try:raw=self.codec.decode(row[5])
        except Unknown as error:self.bad(str(error))
        return super().canonical((*row[:5],raw,row[6]))

    def get(self,key,kind):
        row=self.execute('SELECT kind,raw_length,bytes FROM payload WHERE hash=?',(key,)).fetchone()
        if row is None or row[0]!=kind:self.bad('missing/wrong payload')
        try:raw=self.codec.decode(row[2])
        except Unknown as error:self.bad(str(error))
        if len(raw)!=row[1] or sha(bytes([kind])+raw)!=key:self.bad('decoded payload commitment')
        return raw

    def publish(self,*args,**kwargs):raise Unknown('offline codec candidate; no publication API')
    def reconcile(self,*args,**kwargs):raise Unknown('offline codec candidate; no publication API')
    def move(self,*args,**kwargs):raise Unknown('offline codec candidate; no publication API')


def corpus(db):
    samples={}
    for kind,length,raw in db.execute('SELECT kind,raw_length,bytes FROM payload ORDER BY local'):
        samples.setdefault('payload-'+str(kind),[]).append(decompress(raw,length))
    samples['receipt']=[row[0] for row in db.execute('SELECT body FROM receipt ORDER BY seq')]
    # Training uses unique samples. Evaluation keeps actual retained rows and reports overlap.
    return {family:list(dict.fromkeys(rows)) for family,rows in samples.items()}


def catalog(codec):
    return frame([b'Z1',*[value for ident,raw in sorted(codec.dictionary.items()) for value in (ident,sha(raw),len(raw))]])


def run(training,source,output,library,trained_source=None):
    output.mkdir(parents=True,exist_ok=False)
    origin=sqlite3.connect(source.resolve().as_uri()+'?mode=ro',uri=True);origin.execute('BEGIN')
    report={'status':'running','library_sha256':sha(library.read_bytes()).hex(),'training_source':str(training),'evaluation_source':str(source),
            'candidates':{}}
    trainer=Zstd(library);report['zstd_runtime']=trainer.lib.ZSTD_versionString().decode();trained={};training_metrics={}
    families={}
    if trained_source is None:
        train=sqlite3.connect(training.resolve().as_uri()+'?mode=ro',uri=True);samples=corpus(train);train.close()
        all_samples=list(dict.fromkeys(raw for rows in samples.values() for raw in rows))
        for label,rows,capacity in [('global8',all_samples,8192),('global32',all_samples,32768)]:
            start=time.perf_counter_ns();trained[label]=trainer.train(rows,capacity)
            training_metrics[label]={'ns':time.perf_counter_ns()-start,'bytes':len(trained[label]),'samples':len(rows)}
        for family,rows in samples.items():
            capacity=next((n for n in [8192,4096,1024,512] if len(rows)>=64 and sum(map(len,rows))>=n*100),None)
            if capacity is None:training_metrics['family-'+family]={'not_trained':'fewer than 64 unique samples or less than 100x dictionary bytes'};continue
            label='family-'+family;start=time.perf_counter_ns();trained[label]=trainer.train(rows,capacity);families[family]=label
            training_metrics[label]={'ns':time.perf_counter_ns()-start,'bytes':len(trained[label]),'samples':len(rows)}
        training_info={'source':str(training),'source_sha256':sha(training.read_bytes()).hex(),
            'library_sha256':report['library_sha256'],'runtime':report['zstd_runtime'],
            'families':families,'metrics':training_metrics,
            'samples':{family:{'unique_samples':len(rows),'raw_bytes':sum(map(len,rows)),
                               'sha256':[sha(raw).hex() for raw in rows]} for family,rows in samples.items()}}
    else:
        training_info=json.loads((trained_source/'manifest.json').read_text());families=training_info['families'];training_metrics=training_info['metrics']
        for name,digest in training_info['dictionaries'].items():
            raw=(trained_source/(name+'.dict')).read_bytes()
            if sha(raw).hex()!=digest:raise Unknown('external training dictionary checksum')
            trained[name]=raw
        report['training_source']=training_info['source']
    export=output/'trained';export.mkdir();training_info['dictionaries']={name:sha(raw).hex() for name,raw in trained.items()}
    for name,raw in trained.items():(export/(name+'.dict')).write_bytes(raw)
    (export/'manifest.json').write_text(json.dumps(training_info,indent=2)+'\n')
    report['training']={family:{k:v for k,v in value.items() if k!='sha256'} for family,value in training_info['samples'].items()}
    report['training_manifest_sha256']=sha((export/'manifest.json').read_bytes()).hex()
    sample_hashes={family:set(v['sha256']) for family,v in training_info['samples'].items()}
    trainer.close();report['dictionary_training']=training_metrics
    instance=unframe(origin.execute('SELECT body FROM authority').fetchone()[0])[1]
    baseline=PackedStore(source,instance,'buckets');baseline.open_verified();reference=logical_manifest(baseline)
    reference_cold=sha(checkpoint_bytes(baseline.cold(baseline.meta[6]))).hex();baseline.close()
    definitions=[('zlib6-copy',0,None),('zstd3-plain',3,None),('zstd9-plain',9,None),
                 ('zstd3-global8',3,'global8'),('zstd3-global32',3,'global32'),('zstd9-global8',9,'global8'),('zstd3-family',3,'family')]
    for label,level,profile in definitions:
        folder=output/label;folder.mkdir();dictionary_dir=folder/'dictionaries';dictionary_dir.mkdir();codec=Zstd(library);ids={}
        selected=list(families.values()) if profile=='family' else ([] if profile is None else [profile])
        for name in selected:
            raw=trained[name];ids[name]=codec.load(raw,level);(dictionary_dir/(sha(raw).hex()+'.dict')).write_bytes(raw)
        path=folder/'history.sqlite';dest=sqlite3.connect(path)
        for ddl, in origin.execute("SELECT sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY type='index',name"):
            dest.execute(ddl)
        if level:dest.execute(CATALOG_SCHEMA)
        encoding=[];decoding=[];kinds={};raw_count=0;overlap=0
        for table,order in [('payload','local'),('receipt','seq'),('bucket','id'),('branch','id'),('authority','singleton')]:
            for row in origin.execute('SELECT * FROM '+table+' ORDER BY '+order):
                row=list(row)
                if table in ('payload','receipt'):
                    family='payload-'+str(row[2]) if table=='payload' else 'receipt'
                    raw=decompress(row[4],row[3]) if table=='payload' else row[5]
                    overlap+=int(sha(raw).hex() in sample_hashes.get(family,set()))
                    data=row[4] if table=='payload' else raw
                    if level:
                        name=families.get(family) if profile=='family' else profile;ident=ids.get(name,0)
                        start=time.perf_counter_ns();data=codec.encode(raw,level,ident);encoding.append(time.perf_counter_ns()-start)
                        start=time.perf_counter_ns();restored=codec.decode(data);decoding.append(time.perf_counter_ns()-start)
                        if restored!=raw:raise AssertionError('codec roundtrip lost authority')
                        raw_count+=int(data[0]==0)
                    kinds.setdefault(family,{'rows':0,'raw_bytes':0,'stored_bytes':0});group=kinds[family]
                    group['rows']+=1;group['raw_bytes']+=len(raw);group['stored_bytes']+=len(data)
                    row[4 if table=='payload' else 5]=data
                elif table=='authority' and level:
                    body=unframe(row[1]);body[0]=b'G3Z1'+sha(catalog(codec));row[1]=frame(body);row[2]=sha(b'M'+row[1])
                dest.execute('INSERT INTO '+table+' VALUES ('+','.join('?' for _ in row)+')',row)
        if level:
            body=catalog(codec);dest.execute('INSERT INTO codec VALUES (1,?,?)',(body,sha(body)))
        dest.commit();dest.close()
        s=CodecStore(path,instance,'buckets',codec=codec) if level else PackedStore(path,instance,'buckets')
        opened=s.open_verified();audit=s.audit();actual=logical_manifest(s)
        # Payload manifest includes compressed bytes; compare canonical receipts and every decoded payload instead.
        if actual['receipt']!=reference['receipt']:raise AssertionError('canonical receipts changed')
        for key,kind in s.db.execute('SELECT hash,kind FROM payload'):s.get(key,kind)
        cold=sha(checkpoint_bytes(s.cold(s.meta[6]))).hex()
        if cold!=reference_cold:raise AssertionError('cold reconstruction changed')
        s.db.execute('PRAGMA wal_checkpoint(TRUNCATE)');dictionary_bytes=sum(p.stat().st_size for p in dictionary_dir.iterdir())
        dictionary_alloc=sum(allocation(p).get('allocated_bytes',p.stat().st_size) for p in dictionary_dir.iterdir())
        payload_bytes=sum(v['stored_bytes'] for k,v in kinds.items() if k.startswith('payload-'))
        size=allocation(path);total=size['file_bytes']+dictionary_bytes
        result={'families':kinds,'database':size,'dictionary_bytes':dictionary_bytes,'dictionary_allocated_bytes':dictionary_alloc,
                'total_file_bytes_with_dictionary':total,'total_allocated_bytes_with_dictionary':size.get('allocated_bytes',size['file_bytes'])+dictionary_alloc,
                'compressed_payload_bytes':payload_bytes,'nonpayload_and_dictionary_over_compressed':(total-payload_bytes)/payload_bytes,
                'codec_memory':codec.memory(),'encoding_ns':distribution(encoding),'decoding_ns':distribution(decoding),
                'open':opened,'audit':audit,'pages':pages(s.db),'canonical_receipts':actual['receipt'],'cold_sha256':cold,
                'explicit_raw_blocks':raw_count,'training_exact_sample_overlaps':overlap,'status':'passed'}
        report['candidates'][label]=result;s.close();codec.close();print(label,total,flush=True)
    origin.close();report['status']='passed';(output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('training',type=Path);p.add_argument('source',type=Path)
    p.add_argument('output',type=Path);p.add_argument('--library',type=Path,required=True)
    p.add_argument('--trained',type=Path);a=p.parse_args();run(a.training,a.source,a.output,a.library,a.trained)
