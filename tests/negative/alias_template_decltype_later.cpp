template <class T> using A = decltype(later(T()));
int later(int);
using B = A<int>;
int main() {}
