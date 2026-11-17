#include <cstdio>
struct Pad { virtual ~Pad() {} int p; };
struct A { virtual ~A() {} int id; A():id(42){} virtual A* self(){ return this; } virtual A& ref(){ return *this; } };
struct B : Pad, A { int bb; B():bb(2){} B* self() override { return this; } B& ref() override { return *this; } };
int main(){
  B b; A* a = static_cast<A*>(&b);
  A* (A::* ps)() = &A::self;
  A& (A::* pr)() = &A::ref;
  A* r = (a->*ps)();
  A& rr = (a->*pr)();
  std::printf("%d %d\n", r == a, &rr == a);
  return 0;
}
