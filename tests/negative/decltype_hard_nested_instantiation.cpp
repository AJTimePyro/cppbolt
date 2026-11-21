struct Bad {};
template <typename T> struct Traits { using type = typename T::type; };
template <typename T> auto f(T x) -> decltype(sizeof(typename Traits<T>::type)) { return 1; }
int f(...) { return 2; }
int main() { Bad b; return f(b); }
