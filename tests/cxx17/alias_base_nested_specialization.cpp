#include <iostream>
template <typename T> struct Wrapper { template <typename U> struct Inner { U x; Inner(U v):x(v){} virtual ~Inner(){} }; };
using Alias = Wrapper<int>::Inner<double>;
struct D : Alias { D():Alias(9.5){} };
int main(){ D d; std::cout << d.x << "\n"; }
