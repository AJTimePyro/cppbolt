#include <cstdio>

void takes_str(const char* s) {
    std::printf("decay: %s\n", s);
}

struct OverloadTarget {
    int val;
    OverloadTarget(int* p) { val = (p == nullptr) ? 10 : 20; }
    int member_ptr(int* p) { return (p == nullptr) ? 30 : 40; }
};

int main() {
    int* p1 = 0;
    int* p2 = 0u;
    int val = 123;
    int* p = &val;
    const int* q = p;
    std::printf("p to const q: %d\n", *q);

    int** pp = &p;
    const int* const* ccpp = pp;
    std::printf("multi-level allowed: %d\n", **ccpp);

    int*** p3 = nullptr;
    int** const* q3 = p3;
    if (q3 != nullptr) return 203;

    void* vp1 = nullptr;
    int* np = nullptr;
    std::nullptr_t np0 = 0;
#if defined(__GNUC__) || defined(__clang__)
    bool b(nullptr);
#else
    bool b = nullptr;
#endif
    std::printf("bool from nullptr: %d\n", b ? 1 : 0);

    void* vp2 = p;
    int* back = static_cast<int*>(vp2);
    std::printf("static_cast: %d\n", *back);

    const char* str = "hello";
    std::printf("str: %s, sizeof: %d\n", str, (int)sizeof("hello"));
    takes_str("hello");

    auto auto_str = "hello";
    std::printf("auto ptr decay: %d\n", sizeof(auto_str) == sizeof(const char*) ? 1 : 0);

    OverloadTarget ot(0);
    std::printf("ot: %d, %d\n", ot.val, ot.member_ptr(0));

    return 0;
}
