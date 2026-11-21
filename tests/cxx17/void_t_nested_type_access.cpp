#include <cstdio>
#include <type_traits>

struct HasPrivate {
private:
    typedef int type;
};
struct HasPublic { typedef int type; };
struct B1 { typedef int type; };
struct B2 { typedef char type; };
struct Ambiguous : B1, B2 {};

template <class T, class = void>
struct Detect { static const int value = 0; };

template <class T>
struct Detect<T, std::void_t<typename T::type>> { static const int value = 1; };

int main() {
    printf("%d %d %d\n", Detect<HasPublic>::value,
           Detect<HasPrivate>::value, Detect<Ambiguous>::value);
    return 0;
}
