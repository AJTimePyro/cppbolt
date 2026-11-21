namespace N { void f(int) {} }
template <class T> void caller(T) { f(1); }
using N::f;
int main() { caller(1); }
