#include <iostream>
struct B { virtual ~B(){} virtual void g() & { std::cout<<"Bl"; } virtual void g() && { std::cout<<"Br"; } };
struct D : B { void g() & override { std::cout<<"Dl"; } void g() && override { std::cout<<"Dr"; } };
int main(){ D d; d.g(); static_cast<D&&>(d).g(); std::cout<<"\n"; }
