// The compiler does not support templates, so a built-in header that uses
// them fails while it is being compiled.  The diagnostic must name the
// header and the line inside it, not the file that included it.
#include <initializer_list>

int main() {
    return 0;
}
