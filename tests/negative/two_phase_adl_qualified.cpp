namespace N { struct S {}; void f(S) {} }
namespace Q {}
int main() { Q::f(N::S{}); }
