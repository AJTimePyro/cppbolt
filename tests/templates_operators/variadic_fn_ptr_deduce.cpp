#include <cstdio>

template <typename R, typename... Args>
int arity(R (*)(Args...)) {
    return sizeof...(Args);
}

template <typename R, typename... Args>
R invoke_call(R (*fn)(Args...), Args... args) {
    return fn(args...);
}

template <typename R, typename First, typename... Rest>
R invoke_split(R (*fn)(First, Rest...), First first, Rest... rest) {
    return fn(first, rest...);
}

int zero() {
    return 42;
}

int one(int x) {
    return x * 2;
}

int two(int a, int b) {
    return a + b;
}

const char* three(int a, double b, const char* c) {
    return c;
}

int main() {
    std::printf("zero: %d\n", arity(zero));
    std::printf("one: %d\n", arity(one));
    std::printf("two: %d\n", arity(two));
    std::printf("three: %d\n", arity(three));

    int r0 = invoke_call(zero);
    int r1 = invoke_call(one, 10);
    int r2 = invoke_call(two, 20, 30);
    const char* r3 = invoke_call(three, 1, 2.0, "ok");
    int r4 = invoke_split(two, 100, 200);

    std::printf("r0=%d r1=%d r2=%d r3=%s r4=%d\n", r0, r1, r2, r3, r4);
    return 0;
}
