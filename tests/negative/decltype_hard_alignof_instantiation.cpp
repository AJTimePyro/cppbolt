struct Bad {};
template <typename T> struct Traits { using type = typename T::type; };
template <typename T> auto f(T) -> decltype(alignof(typename Traits<T>::type)) { return 1; }
int main() { Bad b; return f(b); }
