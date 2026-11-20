struct S { static constexpr int value = 1; };

static_assert(S::missing == 1, "unknown member");

int main() { return 0; }
