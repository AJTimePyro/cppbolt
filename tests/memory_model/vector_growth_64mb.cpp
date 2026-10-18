#include <cstdio>
#include <cstdlib>

#ifdef __CPP_WASM__
extern "C" int __builtin_wasm_memory_size(int);
inline int get_current_pages() { return __builtin_wasm_memory_size(0); }
#else
static int s_bump = 8454144;
static int s_bytes = 256 * 65536;
inline int get_current_pages() { return s_bytes / 65536; }
void* tracked_malloc(int sz) {
    int sz_alloc = (sz + 7) & ~7;
    if (sz_alloc < 8) sz_alloc = 8;
    s_bump += sz_alloc + 8;
    if (s_bump > s_bytes) {
        int needed_pages = (s_bump - s_bytes + 65535) / 65536;
        s_bytes += needed_pages * 65536;
    }
    return malloc(sz);
}
#define malloc tracked_malloc
#endif

int main() {
    int cap = 1024; // 1 KB
    char* buf = (char*)malloc(cap);
    buf[0] = 1;

    int peak_pages = get_current_pages();

    printf("Start: cap = %d bytes, pages = %d\n", cap, peak_pages);

    while (cap < 64 * 1024 * 1024) {
        int new_cap = cap * 2;
        char* new_buf = (char*)malloc(new_cap);
        if (!new_buf) {
            printf("Alloc failed at %d\n", new_cap);
            return 1;
        }

        // Copy pattern
        new_buf[0] = buf[0];
        new_buf[cap - 1] = (char)(cap & 0x7f);
        new_buf[new_cap - 1] = (char)(new_cap & 0x7f);

        free(buf);
        buf = new_buf;
        cap = new_cap;

        int cur_pages = get_current_pages();
        if (cur_pages > peak_pages) {
            peak_pages = cur_pages;
        }

        if (cap >= 4 * 1024 * 1024) {
            printf("Live: %d bytes (%d MB), Peak pages: %d (approx %d MB)\n",
                   cap, cap / (1024 * 1024), peak_pages, (peak_pages * 64) / 1024);
        }
    }

    free(buf);
    printf("Final peak pages: %d vs final live bytes: %d\n", peak_pages, 0);
    return 0;
}
