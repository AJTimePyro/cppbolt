template <class T> struct Traits { typedef T type; };
template <class T> struct Traits<T*> {};

using Invalid = Traits<int*>::type;
int main() { return 0; }
