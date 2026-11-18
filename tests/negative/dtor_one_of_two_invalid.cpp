template<class T> struct Good { T v; Good(T x):v(x){} ~Good(){ (void)v; } };
template<class T> struct Bad { T v; Bad(T x):v(x){} ~Bad(){ int* p = v; (void)p; } };
int main(){ Good<int> g(1); Bad<int> b(2); }
