#include <cstdio>

template <class T>
struct Holder {
    typedef typename T::type value_type;
    static const int n = sizeof(typename T::type);
};

struct HasInt { typedef int type; };

int main() {
    printf("%d\n", Holder<HasInt>::n);
    return 0;
}
