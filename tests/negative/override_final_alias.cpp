struct Base { virtual void f() final {} };
using Alias = Base;
struct D : Alias { void f() override {} };
int main(){}
