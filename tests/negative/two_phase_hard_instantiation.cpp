struct Bad {};
template <typename T> struct Traits { using type = typename T::type; };
void g(int) {}
template <typename T> void f(T) { g(typename Traits<T>::type{}); }
int main() { f(Bad{}); }
