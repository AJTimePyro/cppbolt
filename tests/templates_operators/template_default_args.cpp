#include <iostream>

template <typename T, typename U = int>
struct PairDef {
    T first;
    U second;
    PairDef(T a, U b) : first(a), second(b) {}
    void print() { std::cout << first << " " << second << std::endl; }
};

template <typename T = int>
T default_val() {
    return 42;
}

int main() {
    PairDef<const char*> p("answer", 42);
    p.print();
    std::cout << default_val<>() << std::endl;
    return 0;
}
