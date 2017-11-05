typedef unsigned long long int 	uint64_t;
typedef int boolean_t;
struct nvlist;

typedef struct nvlist nvlist_t;
enum lzc_send_flags {
        LZC_SEND_FLAG_EMBED_DATA =  1,
        LZC_SEND_FLAG_LARGE_BLOCK = 2
    };
    typedef enum {
        DMU_OST_NONE,
        DMU_OST_META,
        DMU_OST_ZFS,
        DMU_OST_ZVOL,
        DMU_OST_OTHER,
        DMU_OST_ANY,
        DMU_OST_NUMTYPES
    } dmu_objset_type_t;
    int libzfs_core_init(void);
    void libzfs_core_fini(void);
    int lzc_snapshot(nvlist_t *, nvlist_t *, nvlist_t **);
    int lzc_create(const char *, dmu_objset_type_t, nvlist_t *);
    int lzc_clone(const char *, const char *, nvlist_t *);
    int lzc_destroy_snaps(nvlist_t *, boolean_t, nvlist_t **);
    int lzc_bookmark(nvlist_t *, nvlist_t **);
    int lzc_get_bookmarks(const char *, nvlist_t *, nvlist_t **);
    int lzc_destroy_bookmarks(nvlist_t *, nvlist_t **);
    int lzc_snaprange_space(const char *, const char *, uint64_t *);
    int lzc_hold(nvlist_t *, int, nvlist_t **);
    int lzc_release(nvlist_t *, nvlist_t **);
    int lzc_get_holds(const char *, nvlist_t **);
    int lzc_send(const char *, const char *, int, enum lzc_send_flags);
    int lzc_receive(const char *, nvlist_t *, const char *, boolean_t, int);
    int lzc_send_space(const char *, const char *, uint64_t *);
    boolean_t lzc_exists(const char *);
    int lzc_rollback(const char *, char *, int);
    int lzc_promote(const char *, nvlist_t *, nvlist_t **);
    int lzc_rename(const char *, const char *, nvlist_t *, char **);
    int lzc_destroy_one(const char *fsname, nvlist_t *);
    int lzc_inherit(const char *fsname, const char *name, nvlist_t *);
    int lzc_set_props(const char *, nvlist_t *, nvlist_t *, nvlist_t *);
    int lzc_list (const char *, nvlist_t *);
