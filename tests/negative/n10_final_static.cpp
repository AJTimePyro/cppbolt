struct B { static int f() final { return 1; } };
int main() { return B::f(); }
