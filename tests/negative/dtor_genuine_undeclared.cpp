template<class T> struct B { T v; B(T x):v(x){} ~B(){ never_declared(); } };
int main(){ B<int> b(1); }
