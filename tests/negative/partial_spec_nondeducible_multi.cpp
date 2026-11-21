template<class T, class U>
struct S {};

template<class T, class U>
struct S<T, typename U::type> {};

int main() { return 0; }
