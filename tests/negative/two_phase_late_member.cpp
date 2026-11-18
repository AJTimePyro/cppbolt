struct A { template <class T> void call(T t) { g(t); } };
void g(int) {}
int main() { A a; a.call(1); }
