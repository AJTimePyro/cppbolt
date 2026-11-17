#include <iostream>
struct B { virtual ~B(){ std::cout<<"~B "; } };
struct D : B { ~D() override { std::cout<<"~D "; } };
int main(){ D d; B* p=&d; p->~B(); std::cout<<"\n"; }
