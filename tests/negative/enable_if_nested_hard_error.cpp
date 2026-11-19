template<class T> struct Inner { using type = typename T::missing; };
template<class T> void f(typename Inner<T>::type) {}

int main() { f<int>(0); }
