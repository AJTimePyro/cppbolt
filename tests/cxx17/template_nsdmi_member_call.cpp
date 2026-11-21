#include <cstdio>

template<class T>
struct Box {
    T value = T(3);

    T twice() {
        return value + value;
    }

    T other = twice();
};

// A method declared after the NSDMI that calls it.
template<class T>
struct Late {
    T value = T(4);

    T later();

    T other = later();
};

template<class T>
T Late<T>::later() {
    return value * 2;
}

Box<int> box;

int main() {
    Box<long long> wide;
    Late<int> late;
    Box<int> local;
    printf("%d %d | %lld %lld | %d %d\n",
           box.value, box.other, wide.value, wide.other,
           late.value, late.other);
    printf("local %d %d\n", local.value, local.other);
    return 0;
}
