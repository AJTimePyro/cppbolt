#include <cstdio>

template <typename... Ts>
struct Outer {
    template <typename... Us>
    struct Inner {
        int a;
        int b;
        void print() {
            std::printf("Outer=%d Inner=%d sum=%d\n",
                        (int)sizeof...(Ts), (int)sizeof...(Us), a + b);
        }
    };
};

template <typename... Ts>
void run_outer() {
    typename Outer<Ts...>::template Inner<short, int, long> v1;
    v1.a = 30;
    v1.b = 40;
    v1.print();
}

template <typename... Us>
void run_inner() {
    Outer<int, double, char>::Inner<Us...> v2;
    v2.a = 100;
    v2.b = 200;
    v2.print();
}

int main() {
    Outer<int, double>::Inner<char, float, void*> v0;
    v0.a = 1;
    v0.b = 2;
    v0.print();

    run_outer<int>();
    run_outer<int, long, short>();
    run_inner<double>();
    run_inner<int, int, int>();
    return 0;
}
