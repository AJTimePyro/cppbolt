#include <iostream>
struct A { virtual ~A(){} int id; A(int i):id(i){} virtual A* get(){ return this; } };
struct B : A { B(int i):A(i){} };
struct C : B { C(int i):B(i){} C* get() override { return this; } };
int main(){ A* a = new C(12); std::cout << a->get()->id << "\n"; delete a; }
