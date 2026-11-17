struct A { int a; };
struct B : private A {};
struct X { virtual ~X(){} virtual A* f(){return nullptr;} };
struct Y : X { B* f() override {return nullptr;} };
int main(){ return 0; }
