#include <iostream>
struct R { ~R(){ std::cout << "~R "; } };
template <typename T> struct Base { R r; Base(){} };
struct D : Base<int> { ~D(){ std::cout << "~D "; } };
int main(){ { D d; } std::cout << "\n"; }
