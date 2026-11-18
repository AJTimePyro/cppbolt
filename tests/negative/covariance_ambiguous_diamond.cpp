struct A { int a; };
struct B1 : A {};
struct B2 : A {};
struct C : B1, B2 {};
struct X { virtual ~X(){} virtual A* f(){return nullptr;} };
struct Y : X { C* f() override {return nullptr;} };
int main(){ return 0; }
