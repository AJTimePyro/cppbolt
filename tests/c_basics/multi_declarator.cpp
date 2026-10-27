// Several declarators in one declaration: every declarator applies its own
// pointer, reference and array suffixes to the shared type specifiers.
#include <cstdio>

int g_a = 1, g_b = 2;
int g_arr[2] = {3, 4}, g_c = 5;
int *g_ptr, g_plain;

struct Pair {
    int x, y;
    int *link, plain;
    int arr[2], tail;

    Pair(int v) : x(v), y(v + 1) {}
};

int count_calls() {
    static int calls = 0, step = 2;
    calls = calls + step;
    return calls;
}

int read_const_ptr(int * const p) {
    return *p;
}

int main() {
    int a, b;
    a = 10;
    b = 20;

    int c = 7, d = c + 1, e;

    int *p, q;
    q = 30;
    p = &q;

    int arr[2] = {40, 50}, last = 60;

    int total = 0;
    for (int i = 0, n = 3; i < n; i = i + 1) {
        total = total + i;
    }

    const char *s1 = "left", *s2 = "right";

    g_ptr = &g_plain;
    *g_ptr = 70;

    Pair one(1), two(5);
    two.plain = 11;
    two.link = &q;
    two.arr[1] = 12;
    two.tail = 13;
    *two.link = 14;

    std::printf("%d %d\n", g_a, g_b);
    std::printf("%d %d %d\n", g_arr[1], g_c, g_plain);
    std::printf("%d %d %d %d\n", a, b, c, d);
    e = 0;
    std::printf("%d %d %d\n", *p, q, e);
    std::printf("%d %d %d\n", arr[0], arr[1], last);
    std::printf("%d\n", total);
    std::printf("%s %s\n", s1, s2);
    int first = count_calls();
    int second = count_calls();
    std::printf("%d %d\n", first, second);
    std::printf("%d %d %d %d\n", one.x, one.y, two.x, two.y);
    std::printf("%d %d %d %d\n", two.plain, *two.link, two.arr[1], two.tail);

    int base_val = 100;
    int &ref_val = base_val, scalar_val = 200;
    ref_val = ref_val + 5;
    std::printf("%d %d\n", base_val, scalar_val);

    int * const const_p = &scalar_val, *plain_p = &base_val;
    std::printf("%d %d\n", read_const_ptr(const_p), *plain_p);
    return 0;
}
