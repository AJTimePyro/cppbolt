// The assertion below must be rejected at compile time.
static_assert(1 == 2, "deliberate failure");
int main() { return 0; }
