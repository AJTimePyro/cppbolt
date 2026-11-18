#include <iostream>
struct R { ~R(){ std::cout << "~R "; } };
template <typename T> struct Base { R r; ~Base(){ std::cout << "~Base "; } };
using Alias = Base<int>;
struct Mid : Alias { ~Mid(){ std::cout << "~Mid "; } };
struct Leaf : Mid { ~Leaf(){ std::cout << "~Leaf "; } };
int main(){ { Leaf l; } std::cout << "\n"; }
