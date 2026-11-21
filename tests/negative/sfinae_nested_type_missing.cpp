// A missing nested type discards the only candidate.
struct NoType {};
template <typename T> int f(typename T::type*) { return 1; }
int main() { return f<NoType>(nullptr); }
