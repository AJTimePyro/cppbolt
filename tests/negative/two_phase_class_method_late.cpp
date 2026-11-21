template <class T> struct A { void call() { g(1); } };
void g(int) {}
int main() { A<int> a; a.call(); }
