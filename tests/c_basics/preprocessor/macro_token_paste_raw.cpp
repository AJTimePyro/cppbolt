#include <cstdio>

#define PREFIX item
#define SUFFIX val
#define CONCAT_RAW(a, b) a##b
#define CONCAT_EXP(a, b) CONCAT_RAW(a, b)

#define STR(x) #x
#define FOO 42
#define RESCAN_MAKE() FO##O

#define RESET_ME 99
#undef RESET_ME

int main() {
    // 1. Raw token pasting does not pre-expand operands of ##
    int CONCAT_RAW(PREFIX, SUFFIX) = 10;
    std::printf("PREFIXSUFFIX=%d\n", PREFIXSUFFIX);

    // 2. Normal argument expansion occurs when not adjacent to ##
    int CONCAT_EXP(PREFIX, SUFFIX) = 20;
    std::printf("itemval=%d\n", itemval);

    // 3. Stringification condenses whitespace
    std::printf("STR=%s\n", STR( a   +   b ));

    // 4. Token pasting result is rescanned
    std::printf("RESCAN=%d\n", RESCAN_MAKE());

    // 5. #undef directive causes macro to be forgotten
#ifdef RESET_ME
    std::printf("FAIL: RESET_ME still defined\n");
#else
    std::printf("UNDEF_OK\n");
#endif

    return 0;
}
