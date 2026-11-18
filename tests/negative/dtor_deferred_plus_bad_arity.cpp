void one(int);
template<class T> struct B { T v; B(T x):v(x){} ~B(){ one(v, v); } };
void one(int x){ (void)x; }
int main(){ B<int> b(1); }
