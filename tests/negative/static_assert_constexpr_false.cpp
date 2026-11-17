// A named constexpr false must still fail the static assertion.
constexpr bool bad = false;
static_assert(bad, "must fail");
int main() { return 0; }
