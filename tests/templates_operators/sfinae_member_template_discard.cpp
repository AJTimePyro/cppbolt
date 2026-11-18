#include <cstdio>

struct S {
    // The first candidate's array bound becomes negative during substitution.
    template <typename T> int pick(T x, char (*p)[sizeof(T) - 100]) { return 1; }
    template <typename T> int pick(T x, T* p) { return 2; }
};

int main() {
    S s;
    std::printf("%d\n", s.pick(1, (int*)0));
    return 0;
}
