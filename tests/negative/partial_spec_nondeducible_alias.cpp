template<class...> using void_t = void;

template<class U>
struct S {};

template<class U>
struct S<void_t<typename U::type>> {};

int main() { return 0; }
