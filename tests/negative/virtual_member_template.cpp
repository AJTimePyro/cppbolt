struct B { template <typename T> virtual T f(T v) { return v; } };
int main() { B b; return b.f(1); }
