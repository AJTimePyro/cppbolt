template <class T> void caller(T t) { f(t); }
template <class T> void f(T) {}
int main() { caller(1); }
