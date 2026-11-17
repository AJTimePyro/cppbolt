#include <iostream>
struct Pad { virtual ~Pad(){} int p; };
struct A { virtual ~A(){} int id; virtual A& self(){ return *this; } };
struct B : Pad, A { B* q; B& self() override { return *this; } };
int main(){ B b; A& a = static_cast<A&>(b); A& r = a.self(); std::cout << (&r==&a) << "\n"; }
