#include <cstdio>

template <typename T1, typename T2>
struct MyPair {
    T1 first;
    T2 second;
    MyPair(T1 a, T2 b) : first(a), second(b) {}
};

template <typename T1, typename T2>
MyPair<T1, T2> make_my_pair(T1 a, T2 b) {
    return MyPair<T1, T2>(a, b);
}

int main() {
    MyPair<int, int> p1 = make_my_pair(100, 200);
    std::printf("p1: (%d, %d)\n", p1.first, p1.second);
    return 0;
}
