#include <cstdio>

namespace A {
struct X {};
void by_array(X*) { std::printf("A::by_array\n"); }
}
namespace B {
struct Y {};
void by_function(void (*)(Y)) { std::printf("B::by_function\n"); }
}

void callback(B::Y) {}

int main() {
    A::X values[2];
    by_array(values);
    by_function(callback);
    return 0;
}
