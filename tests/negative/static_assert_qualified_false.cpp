struct S { static constexpr int value = 1; };

static_assert(S::value == 2, "qualified member mismatch");

int main() { return 0; }
