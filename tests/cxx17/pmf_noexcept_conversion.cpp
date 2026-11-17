#include <cstdio>

struct S {
    int x;
    int ne(int n) noexcept { return n + 1; }
    int ne_c(int n) const noexcept { return n + 2; }
    int plain(int n) { return n + 3; }
};

int main() {
    S s{};
    int (S::* pne)(int) noexcept = &S::ne;
    int (S::* pplain)(int) = pne;          // noexcept -> throwing: allowed
    int (S::* pplain2)(int) = &S::ne;      // also allowed directly
    int (S::* pc)(int) const noexcept = &S::ne_c;
    int (S::* pcplain)(int) const = pc;    // cv preserved, noexcept dropped
    int (S::* pthrow)(int) = &S::plain;
    std::printf("%d %d %d %d %d\n", (s.*pplain)(1), (s.*pplain2)(2),
                (s.*pcplain)(3), (s.*pthrow)(4), (s.*pne)(5));
    return 0;
}
