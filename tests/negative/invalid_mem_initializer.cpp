struct Base { int x; Base(int v):x(v){} };
struct Derived : Base { Derived() : NotABase(42) {} };
int main(){ Derived d; return d.x; }
