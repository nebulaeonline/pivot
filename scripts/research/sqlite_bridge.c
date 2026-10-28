/* Research-only SQLite connection, allocator and delegating VFS instrumentation.
 * SQLite public symbols are static: no accidental system-library interposition.
 * Counters/fault controls assume this private harness's single calling thread. */
#define SQLITE_API static
#define SQLITE_THREADSAFE 1
#define SQLITE_DEFAULT_MEMSTATUS 1
#define SQLITE_ENABLE_DBSTAT_VTAB 1
#define SQLITE_OMIT_LOAD_EXTENSION 1
#define SQLITE_MAX_MMAP_SIZE 0
#include "sqlite3.c"
#include <stdint.h>
#include <string.h>
#include <time.h>
#ifdef _WIN32
#define PV __declspec(dllexport)
#else
#define PV __attribute__((visibility("default")))
#endif

static sqlite3_vfs *parent;
static sqlite3_vfs observed;
static sqlite3_mem_methods allocator;
static sqlite3_int64 counters[48];
static int fail_write=-1,fail_sync=-1,fail_alloc=-1;
typedef struct ObservedFile { sqlite3_file file; sqlite3_file *real; int kind; } ObservedFile;
#define F(p) ((ObservedFile*)(p))
#define R(p) (F(p)->real)
#define M(p) (R(p)->pMethods)
static sqlite3_int64 clock_ns(void){
#ifdef _WIN32
  LARGE_INTEGER n,f; QueryPerformanceCounter(&n);QueryPerformanceFrequency(&f);
  return (sqlite3_int64)((double)n.QuadPart*1000000000.0/(double)f.QuadPart);
#else
  struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return (sqlite3_int64)t.tv_sec*1000000000+t.tv_nsec;
#endif
}
static int trip(int *n){ if(*n<0)return 0;if(*n==0){*n=-1;return 1;}--*n;return 0; }
static int allocation_refused(void){
  if(fail_alloc<0)return 0;
  if(fail_alloc==0){counters[47]++;return 1;}--fail_alloc;return 0;
}
static void *memory_malloc(int n){return allocation_refused()?0:allocator.xMalloc(n);}
static void *memory_realloc(void *p,int n){return allocation_refused()?0:allocator.xRealloc(p,n);}
static int io_close(sqlite3_file *p){return M(p)->xClose(R(p));}
static int io_read(sqlite3_file *p,void *b,int n,sqlite3_int64 off){
  int base=F(p)->kind*10;counters[base]++;counters[base+1]+=n;return M(p)->xRead(R(p),b,n,off);
}
static int io_write(sqlite3_file *p,const void *b,int n,sqlite3_int64 off){
  int base=F(p)->kind*10;counters[base+2]++;counters[base+3]+=n;
  if(F(p)->kind==1){sqlite3_int64 at=32;if(off>32)at+=((off-32+4119)/4120)*4120;
    while(at+24<=off+n){if(at>=off)counters[40]++;at+=4120;}}
  if(trip(&fail_write)){counters[46]++;return SQLITE_IOERR_WRITE;}
  return M(p)->xWrite(R(p),b,n,off);
}
static int io_truncate(sqlite3_file *p,sqlite3_int64 n){counters[F(p)->kind*10+8]++;return M(p)->xTruncate(R(p),n);}
static int io_sync(sqlite3_file *p,int flags){
  int base=F(p)->kind*10,rc;sqlite3_int64 start=clock_ns();counters[base+4]++;
  if((flags&15)==SQLITE_SYNC_FULL)counters[base+6]++;else counters[base+7]++;
  if(trip(&fail_sync)){counters[46]++;rc=SQLITE_IOERR_FSYNC;}else rc=M(p)->xSync(R(p),flags);
  counters[base+5]+=clock_ns()-start;return rc;
}
static int io_size(sqlite3_file *p,sqlite3_int64 *n){return M(p)->xFileSize(R(p),n);}
static int io_lock(sqlite3_file *p,int n){return M(p)->xLock(R(p),n);}
static int io_unlock(sqlite3_file *p,int n){return M(p)->xUnlock(R(p),n);}
static int io_reserved(sqlite3_file *p,int *n){return M(p)->xCheckReservedLock(R(p),n);}
static int io_control(sqlite3_file *p,int op,void *a){
  if(op==SQLITE_FCNTL_CKPT_START)counters[44]++;
  if(op==SQLITE_FCNTL_CKPT_DONE)counters[45]++;
  return M(p)->xFileControl(R(p),op,a);
}
static int io_sector(sqlite3_file *p){return M(p)->xSectorSize(R(p));}
static int io_device(sqlite3_file *p){return M(p)->xDeviceCharacteristics(R(p));}
static int io_map(sqlite3_file *p,int a,int b,int c,void volatile **v){return M(p)->xShmMap(R(p),a,b,c,v);}
static int io_shmlock(sqlite3_file *p,int a,int b,int c){return M(p)->xShmLock(R(p),a,b,c);}
static void io_barrier(sqlite3_file *p){M(p)->xShmBarrier(R(p));}
static int io_unmap(sqlite3_file *p,int n){return M(p)->xShmUnmap(R(p),n);}
static int io_fetch(sqlite3_file *p,sqlite3_int64 off,int n,void **v){
  if(M(p)->iVersion<3||!M(p)->xFetch){*v=0;return SQLITE_OK;}return M(p)->xFetch(R(p),off,n,v);
}
static int io_unfetch(sqlite3_file *p,sqlite3_int64 off,void *v){
  return M(p)->iVersion<3||!M(p)->xUnfetch?SQLITE_OK:M(p)->xUnfetch(R(p),off,v);
}
static const sqlite3_io_methods methods={3,io_close,io_read,io_write,io_truncate,io_sync,io_size,io_lock,io_unlock,
 io_reserved,io_control,io_sector,io_device,io_map,io_shmlock,io_barrier,io_unmap,io_fetch,io_unfetch};
static int v_open(sqlite3_vfs *v,sqlite3_filename name,sqlite3_file *p,int flags,int *out){
  int rc;ObservedFile *f=F(p);(void)v;memset(f,0,sizeof(*f));f->real=(sqlite3_file*)((char*)p+sizeof(*f));
  f->kind=(flags&SQLITE_OPEN_WAL)?1:((flags&SQLITE_OPEN_MAIN_DB)?0:2);
  rc=parent->xOpen(parent,name,f->real,flags,out);
  if(rc==SQLITE_OK)f->file.pMethods=&methods;return rc;
}
static int v_delete(sqlite3_vfs *v,const char *n,int s){(void)v;return parent->xDelete(parent,n,s);}
static int v_access(sqlite3_vfs *v,const char *n,int f,int *r){(void)v;return parent->xAccess(parent,n,f,r);}
static int v_path(sqlite3_vfs *v,const char *n,int k,char *r){(void)v;return parent->xFullPathname(parent,n,k,r);}
static void *v_dlopen(sqlite3_vfs *v,const char *n){(void)v;return parent->xDlOpen?parent->xDlOpen(parent,n):0;}
static void v_dlerror(sqlite3_vfs *v,int n,char *r){(void)v;if(parent->xDlError)parent->xDlError(parent,n,r);}
static void (*v_dlsym(sqlite3_vfs *v,void *p,const char *n))(void){(void)v;return parent->xDlSym?parent->xDlSym(parent,p,n):0;}
static void v_dlclose(sqlite3_vfs *v,void *p){(void)v;if(parent->xDlClose)parent->xDlClose(parent,p);}
static int v_random(sqlite3_vfs *v,int n,char *r){(void)v;return parent->xRandomness(parent,n,r);}
static int v_sleep(sqlite3_vfs *v,int n){(void)v;return parent->xSleep(parent,n);}
static int v_time(sqlite3_vfs *v,double *r){(void)v;return parent->xCurrentTime(parent,r);}
static int v_error(sqlite3_vfs *v,int n,char *r){(void)v;return parent->xGetLastError?parent->xGetLastError(parent,n,r):0;}
static int v_time64(sqlite3_vfs *v,sqlite3_int64 *r){(void)v;return parent->xCurrentTimeInt64(parent,r);}
static int v_setcall(sqlite3_vfs *v,const char *n,sqlite3_syscall_ptr p){(void)v;return parent->xSetSystemCall?parent->xSetSystemCall(parent,n,p):SQLITE_NOTFOUND;}
static sqlite3_syscall_ptr v_getcall(sqlite3_vfs *v,const char *n){(void)v;return parent->xGetSystemCall?parent->xGetSystemCall(parent,n):0;}
static const char *v_nextcall(sqlite3_vfs *v,const char *n){(void)v;return parent->xNextSystemCall?parent->xNextSystemCall(parent,n):0;}
static int wal_commit(void *p,sqlite3 *db,const char *name,int n){
  (void)p;(void)db;(void)name;counters[41]++;counters[42]=n;if(n>counters[43])counters[43]=n;return SQLITE_OK;
}
PV int pv_init(void){
  sqlite3_mem_methods wrapped;int rc=sqlite3_config(SQLITE_CONFIG_GETMALLOC,&allocator);if(rc)return rc;
  wrapped=allocator;wrapped.xMalloc=memory_malloc;wrapped.xRealloc=memory_realloc;
  rc=sqlite3_config(SQLITE_CONFIG_MALLOC,&wrapped);if(rc)return rc;rc=sqlite3_initialize();if(rc)return rc;
  parent=sqlite3_vfs_find(0);if(!parent)return SQLITE_ERROR;observed=*parent;
  observed.zName="pivot-observe";observed.pNext=0;observed.szOsFile=sizeof(ObservedFile)+parent->szOsFile;
  observed.xOpen=v_open;observed.xDelete=v_delete;observed.xAccess=v_access;observed.xFullPathname=v_path;
  observed.xDlOpen=v_dlopen;observed.xDlError=v_dlerror;observed.xDlSym=v_dlsym;observed.xDlClose=v_dlclose;
  observed.xRandomness=v_random;observed.xSleep=v_sleep;observed.xCurrentTime=v_time;observed.xGetLastError=v_error;
  observed.xCurrentTimeInt64=v_time64;observed.xSetSystemCall=v_setcall;observed.xGetSystemCall=v_getcall;observed.xNextSystemCall=v_nextcall;
  return sqlite3_vfs_register(&observed,0);
}
PV const char *pv_version(void){return sqlite3_libversion();}
PV const char *pv_source(void){return sqlite3_sourceid();}
PV const char *pv_parent(void){return parent->zName;}
PV int pv_open(const char *name,sqlite3 **db){
  int rc=sqlite3_open_v2(name,db,SQLITE_OPEN_READWRITE|SQLITE_OPEN_CREATE|SQLITE_OPEN_FULLMUTEX,"pivot-observe");
  if(rc==SQLITE_OK){sqlite3_extended_result_codes(*db,1);sqlite3_busy_timeout(*db,200);}return rc;
}
PV int pv_close(sqlite3 *db){return sqlite3_close(db);}
PV const char *pv_error(sqlite3 *db){return sqlite3_errmsg(db);}
PV int pv_errcode(sqlite3 *db){return sqlite3_extended_errcode(db);}
PV int pv_auto(sqlite3 *db){return sqlite3_get_autocommit(db);}
PV int pv_prepare(sqlite3 *db,const char *s,sqlite3_stmt **st){return sqlite3_prepare_v2(db,s,-1,st,0);}
PV int pv_exec(sqlite3 *db,const char *s){return sqlite3_exec(db,s,0,0,0);}
/* Pinned-source introspection only: sample dirty-list length between SQL steps.
 * This is not a public SQLite ABI or a within-statement peak measurement. */
PV int pv_dirty(sqlite3 *db){
  int i,n=0;sqlite3_mutex_enter(sqlite3_db_mutex(db));
  for(i=0;i<db->nDb;i++)if(db->aDb[i].pBt){
    Btree *bt=db->aDb[i].pBt;PgHdr *page;sqlite3BtreeEnter(bt);
    page=sqlite3BtreePager(bt)->pPCache->pDirty;
    while(page){n++;page=page->pDirtyNext;}sqlite3BtreeLeave(bt);
  }
  sqlite3_mutex_leave(sqlite3_db_mutex(db));return n;
}
PV int pv_step(sqlite3_stmt *s){
  int rc=sqlite3_step(s),n=pv_dirty(sqlite3_db_handle(s));
  counters[30]=n;if(n>counters[31])counters[31]=n;return rc;
}
PV int pv_finalize(sqlite3_stmt *s){return sqlite3_finalize(s);}
PV int pv_readonly(sqlite3_stmt *s){return sqlite3_stmt_readonly(s);}
PV int pv_bind_null(sqlite3_stmt *s,int n){return sqlite3_bind_null(s,n);}
PV int pv_bind_int(sqlite3_stmt *s,int n,sqlite3_int64 v){return sqlite3_bind_int64(s,n,v);}
PV int pv_bind_double(sqlite3_stmt *s,int n,double v){return sqlite3_bind_double(s,n,v);}
PV int pv_bind_blob(sqlite3_stmt *s,int n,const void *p,int k){return sqlite3_bind_blob(s,n,p,k,SQLITE_TRANSIENT);}
PV int pv_bind_text(sqlite3_stmt *s,int n,const char *p,int k){return sqlite3_bind_text(s,n,p,k,SQLITE_TRANSIENT);}
PV int pv_columns(sqlite3_stmt *s){return sqlite3_column_count(s);}
PV int pv_type(sqlite3_stmt *s,int n){return sqlite3_column_type(s,n);}
PV sqlite3_int64 pv_int(sqlite3_stmt *s,int n){return sqlite3_column_int64(s,n);}
PV double pv_double(sqlite3_stmt *s,int n){return sqlite3_column_double(s,n);}
PV const void *pv_blob(sqlite3_stmt *s,int n){return sqlite3_column_blob(s,n);}
PV const char *pv_text(sqlite3_stmt *s,int n){return (const char*)sqlite3_column_text(s,n);}
PV int pv_bytes(sqlite3_stmt *s,int n){return sqlite3_column_bytes(s,n);}
PV void pv_observe(sqlite3 *db){sqlite3_wal_hook(db,wal_commit,0);}
PV void pv_counters(sqlite3_int64 *out){memcpy(out,counters,sizeof(counters));}
PV void pv_fault(int write,int sync,int allocation){fail_write=write;fail_sync=sync;fail_alloc=allocation;}
PV sqlite3_int64 pv_limit(sqlite3_int64 n){return sqlite3_hard_heap_limit64(n);}
PV void pv_memory(sqlite3_int64 *out){sqlite3_status64(SQLITE_STATUS_MEMORY_USED,out,out+1,0);sqlite3_status64(SQLITE_STATUS_MALLOC_COUNT,out+2,out+3,0);}
PV int pv_status(sqlite3 *db,int op,sqlite3_int64 *cur,sqlite3_int64 *hi){return sqlite3_db_status64(db,op,cur,hi,0);}
PV int pv_vfs(sqlite3 *db,char *buf,int size){
  char *name=0;int rc=sqlite3_file_control(db,"main",SQLITE_FCNTL_VFSNAME,&name);
  if(rc==SQLITE_OK){sqlite3_snprintf(size,buf,"%s",name);sqlite3_free(name);}return rc;
}
