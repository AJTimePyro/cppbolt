struct X {};
int ov(X);
using A = decltype(ov(1));
int ov(int);
int main() {}
