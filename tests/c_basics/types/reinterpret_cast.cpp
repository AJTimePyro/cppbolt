#include <cstdio>

struct A { int val; };
struct B { int num; };

int main() {
    int x = 0x01020304;
    char* cp = reinterpret_cast<char*>(&x);
    std::printf("byte: %d\n", (int)*cp);

    unsigned long long addr = reinterpret_cast<unsigned long long>(&x);
    int* p2 = reinterpret_cast<int*>(addr);
    std::printf("roundtrip: %d\n", *p2 == 0x01020304 ? 1 : 0);

    A a_inst;
    a_inst.val = 42;
    B* b_ptr = reinterpret_cast<B*>(&a_inst);
    std::printf("unrelated_struct: %d\n", b_ptr->num);

    char& cr = reinterpret_cast<char&>(x);
    std::printf("ref_cast: %d\n", (int)cr);

    cr = 0x20;
    std::printf("mutated_ref: %d\n", (int)(x & 0xff));

    reinterpret_cast<int&>(x) = 0x11223344;
    std::printf("direct_assign: %d\n", x == 0x11223344 ? 1 : 0);

    float f = 1.0f;
    int& fi = reinterpret_cast<int&>(f);
    std::printf("float_bits: %d\n", fi == 0x3f800000 ? 1 : 0);

    return 0;
}
