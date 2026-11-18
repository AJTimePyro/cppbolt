#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){} virtual T get(){return x;} };
using Alias = Base<int>;
struct Mid : Alias { Mid():Alias(10){} };
struct Leaf : Mid { Leaf():Mid(){} int get() override {return x+1;} };
int main(){ Leaf l; std::cout << l.x << " " << l.get() << "\n"; }
