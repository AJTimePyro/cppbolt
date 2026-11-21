namespace N { struct S { friend void hidden(S) {} }; }
int main() { hidden(1); }
