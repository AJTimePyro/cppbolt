struct B { virtual ~B() final {} };
struct D : B {};
struct E : D {};
int main(){ E e; return 0; }
