#include <iostream>
struct Pad { virtual ~Pad(){} int p; Pad():p(1){} };
struct A { virtual ~A(){} int id; A():id(42){} virtual A* self(){ return this; } };
struct B : Pad, A { int bb; B():bb(2){} B* self() override { return this; } };
int main(){ B b; A* a = static_cast<A*>(&b); A* r = a->self(); std::cout << (r==a) << " " << r->id << "\n"; }
