template<class T>
struct S {};

template<class T>
struct S<typename T::type> {};

int main() { return 0; }
