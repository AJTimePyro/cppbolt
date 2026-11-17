struct B { virtual ~B() final {} };
struct D : B {};
int main(){ D d; return 0; }
