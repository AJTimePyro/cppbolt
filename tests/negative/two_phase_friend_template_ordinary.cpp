namespace N { template <class T> struct S { friend void hidden(S) {} }; }
int main() { hidden(1); }
