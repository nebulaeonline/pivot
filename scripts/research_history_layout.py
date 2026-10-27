"""G3 physical-layout candidates preserving all external keys and semantic fields."""
from research_history_compact import CompactStore,SCHEMA,sha,frame


def schema(layout):
    sql=SCHEMA
    if layout=='rowid':
        sql=sql.replace('hash BLOB PRIMARY KEY,kind','hash BLOB PRIMARY KEY NOT NULL,kind',1).replace(') WITHOUT ROWID;',');',1)
    elif layout in ('local','partial'):
        sql=sql.replace('hash BLOB PRIMARY KEY,kind','local INTEGER PRIMARY KEY,hash BLOB UNIQUE NOT NULL,kind',1).replace(') WITHOUT ROWID;',');',1)
    elif layout!='without':raise ValueError('layout')
    if layout=='partial':
        sql=sql.replace('edit INTEGER UNIQUE,body','edit INTEGER,body')
        sql+='CREATE UNIQUE INDEX receipt_edit ON receipt(edit) WHERE edit IS NOT NULL;'
    return sql


class LayoutStore(CompactStore):
    def __init__(self,*args,layout='without',**kwargs):
        self.layout=layout;self.schema_sql=schema(layout)
        super().__init__(*args,**kwargs)

    def put(self,blob):
        # Local physical keys are never persisted in authoritative references.
        self.execute('INSERT OR IGNORE INTO payload(hash,kind,raw_length,bytes) VALUES (?,?,?,?)',blob)
        row=self.execute('SELECT kind,raw_length,bytes FROM payload WHERE hash=?',(blob[0],)).fetchone()
        if row!=blob[1:]:self.bad('payload identity disagreement')
        return blob[0]


def manifest(db):
    """Logical bytes only: exclude local rowids and physical schema, include all authority."""
    result={}
    for name,query in [
        ('payload','SELECT hash,kind,raw_length,bytes FROM payload ORDER BY hash'),
        ('receipt','SELECT * FROM receipt ORDER BY seq'),
        ('bucket','SELECT * FROM bucket ORDER BY id'),
        ('authority','SELECT body,hash FROM authority ORDER BY singleton'),
    ]:
        import hashlib
        h=hashlib.sha256();count=0
        for row in db.execute(query):
            raw=frame(row);h.update(len(raw).to_bytes(8,'little'));h.update(raw);count+=1
        result[name]={'rows':count,'sha256':h.hexdigest()}
    return result
