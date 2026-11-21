template <typename T> auto f(T x) -> decltype(x.member) { return missing_function(x); }
struct S { int member; };
int main() { S s; return f(s); }
