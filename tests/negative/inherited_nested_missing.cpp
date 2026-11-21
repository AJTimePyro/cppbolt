struct Base { typedef int value_type; };
struct Derived : Base {};
using X = Derived::missing;
int main() { return 0; }
