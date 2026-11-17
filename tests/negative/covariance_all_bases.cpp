struct X { int x; };
struct A { virtual ~A(){} virtual A* f(){return this;} };
struct B { virtual ~B(){} virtual X* f(){return nullptr;} };
struct C : A, B { C* f() override {return this;} };
int main(){ return 0; }
