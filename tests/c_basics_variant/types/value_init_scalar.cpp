#include <cstdio>

typedef bool B;
typedef char C;
typedef long long LL;

int main() {
    bool b1 = bool();
    bool b2 = bool(1);
    B b3 = B();
    char c1 = char();
    char c2 = char(65);
    C c3 = C(66);
    LL ll1 = LL();
    LL ll2 = LL(1234567890123LL);
    long long ll3 = LL();
    std::printf("%d %d %d %d %c %c %lld %lld %lld\n",
                (int)b1, (int)b2, (int)b3, (int)c1, c2, c3, ll1, ll2, ll3);
    return 0;
}
