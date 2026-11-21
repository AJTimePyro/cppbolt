// Traits<Bad> is ill-formed while resolving Traits<T>::type, so this is hard.
struct Good { using type = int; };
struct Bad {};
template <typename T> struct Traits { using type = typename T::type; };
template <typename T> int f(typename Traits<T>::type* p) { return 1; }
template <typename T> int f(T*) { return 2; }
int main() { return f<Good>(nullptr) + f<Bad>(nullptr); }
