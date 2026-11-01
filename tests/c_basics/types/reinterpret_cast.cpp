#include <cstdio>

struct A { int val; };
struct B { int num; };

int main() {
    int x = 0x01020304;
    char* cp = reinterpret_cast<char*>(&x);
    std::printf("byte: %d\n", (int)*cp);

    unsigned int addr = reinterpret_cast<unsigned int>(&x);
    int* p2 = reinterpret_cast<int*>(addr);
    std::printf("roundtrip: %d\n", *p2 == 0x01020304 ? 1 : 0);

    A a_inst;
    a_inst.val = 42;
    B* b_ptr = reinterpret_cast<B*>(&a_inst);
    std::printf("unrelated_struct: %d\n", b_ptr->num);

    char& cr = reinterpret_cast<char&>(x);
    std::printf("ref_cast: %d\n", (int)cr);

    return 0;
}
