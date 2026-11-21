#include <cstdio>

namespace N1 { using Val = int; }
namespace N2 { using Val = long long; }

namespace M1 { template <class T> using Box = T; }
namespace M2 { template <class T> using Box = long long; }

int main() {
    N1::Val a = 0;
    N2::Val b = 0;
    M1::Box<int> c = 0;
    M2::Box<int> d = 0;
    printf("%zu %zu %zu %zu\n", sizeof(a), sizeof(b), sizeof(c), sizeof(d));
    return 0;
}
