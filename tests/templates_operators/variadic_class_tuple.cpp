#include <cstdio>

template <typename... Ts>
struct Tuple;

template <>
struct Tuple<> {
    void print() { printf("empty\n"); }
};

template <typename Head, typename... Tail>
struct Tuple<Head, Tail...> : Tuple<Tail...> {
    Head head;
    Tuple(Head h, Tail... tail) : Tuple<Tail...>(tail...), head(h) {}
    void print() {
        printf("item\n");
        Tuple<Tail...>::print();
    }
};

int main() {
    Tuple<int, double> t(1, 2.5);
    t.print();
    return 0;
}
