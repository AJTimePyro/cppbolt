struct A { template <class U> friend void f(A, U) {} };
int main() { f(1); }
