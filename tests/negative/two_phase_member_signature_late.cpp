struct A { template <class T> auto call(T) -> decltype(g(1)) { return 0; } };
int g(int) { return 1; }
int main() { A a; return a.call(1); }
