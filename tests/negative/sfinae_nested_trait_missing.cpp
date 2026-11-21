// A missing nested type in a class template is a substitution failure too.
struct Bad {};
template <typename T> struct Traits {};
template <typename T> int g(typename Traits<T>::type*) { return 1; }
int main() { return g<Bad>(nullptr); }
