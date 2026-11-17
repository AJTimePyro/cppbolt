struct B { virtual ~B() final {} };
struct D : B { ~D() override {} };
int main() { D d; return 0; }
