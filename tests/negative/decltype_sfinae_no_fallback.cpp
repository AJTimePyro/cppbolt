struct No {};
template <typename T> auto f(T x) -> decltype(x.value) { return 1; }
int main() { No n; return f(n); }
