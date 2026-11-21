int f() { return 1; }
static_assert(f(), "runtime call condition");
int main() { return 0; }
