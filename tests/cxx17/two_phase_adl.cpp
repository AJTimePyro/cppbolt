#include <cstdio>

namespace A { struct X {}; }
namespace B {
struct Y {};
void pick(A::X, Y) { std::printf("B::pick\n"); }
}
namespace P {
struct S {};
void by_pointer(S*) { std::printf("P::by_pointer\n"); }
}
namespace Q {
struct Base {};
void via_base(Base*) { std::printf("Q::via_base\n"); }
}
namespace R {
struct Tag {};
template <class T> struct Box {};
void by_argument(R::Box<R::Tag>) { std::printf("R::by_argument\n"); }
}
namespace M {
struct S {};
void merged(S) { std::printf("M::merged\n"); }
}
struct D : Q::Base {};
template <class T> void merged(T) { std::printf("template merged\n"); }

int main() {
    pick(A::X{}, B::Y{});
    P::S ps;
    by_pointer(&ps);
    D d;
    via_base(&d);
    by_argument(R::Box<R::Tag>{});
    merged(M::S{});
    merged(1);
    return 0;
}
