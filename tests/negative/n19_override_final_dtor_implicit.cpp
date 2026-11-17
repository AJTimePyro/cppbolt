struct B { virtual ~B() final {} };
struct D : B { ~D() {} };
int main() { D d; return 0; }
