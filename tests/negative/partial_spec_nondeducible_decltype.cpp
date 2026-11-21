template<class T>
struct S {};

template<class T>
struct S<decltype(T{})> {};

int main() { return 0; }
