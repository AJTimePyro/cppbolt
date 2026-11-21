struct A { typedef int value_type; };
struct B { typedef double value_type; };
struct C : A, B {};
using X = C::value_type;
int main() { return 0; }
