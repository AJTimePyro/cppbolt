#include <cstdio>

// An invalid arrow operand inside the nested decltype rejects the
// specialization through a substitution failure; the primary is selected.

template<class T> struct Box { using type = T; };

struct WithGet { int get() const { return 7; } };

template<class T, class U>
struct N { static constexpr int value = 0; };
template<class T>
struct N<T*, typename Box<decltype(T()->get())>::type> {
    static constexpr int value = 6;
};

int main() {
    std::printf("N<WithGet*,int>=%d\n", N<WithGet*, int>::value);
    return 0;
}
