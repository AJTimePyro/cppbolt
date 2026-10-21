#include <cstdio>

// Every call below uses an implicit conversion that C++ allows.  The
// compiler must accept all of them.  The comment in front of each call
// names the conversion it exercises.

enum Level { Low = 1, High = 2 };

struct Base {
    int id;
    Base(int v) : id(v) {}
};

struct Derived : Base {
    Derived(int v) : Base(v) {}
};

int takes_int(int v) { return v; }
double takes_double(double v) { return v; }
void takes_float(float v) {}
void takes_pointer(int* p) {}
void takes_const_pointer(const int* p) {}
void takes_char_pointer(const char* p) {}
void takes_void_pointer(void* p) {}
void takes_base_ref(Base& b) { b.id = 1; }
void takes_base_value(Base b) { b.id = 1; }
void takes_base_pointer(Base* b) {}
void takes_const_base_pointer(const Base* b) {}

int main() {
    int i = 5;
    double d = 2.5;
    float f = 1.5f;
    char letter = 'x';
    short small = 7;
    long big = 9;
    bool flag = true;
    unsigned int count = 3;
    Level level = High;
    Derived derived(11);
    int array[3] = {1, 2, 3};
    int* ip = &i;
    char buffer[6];
    buffer[0] = 'h';
    char* mutable_text = buffer;

    std::printf("%d\n", takes_int(i));               // exact match
    std::printf("%d\n", takes_int(d));               // 1. double to int
    std::printf("%d\n", takes_int(letter));          // 2. char to int
    std::printf("%d\n", takes_int(small));           // 3. short to int
    std::printf("%d\n", takes_int(big));             // 4. long to int
    std::printf("%d\n", takes_int(flag));            // 5. bool to int
    std::printf("%d\n", takes_int(count));          // 6. unsigned int to int
    std::printf("%d\n", takes_int(level));           // 7. enum to int
    std::printf("%.1f\n", takes_double(i));          // 8. int to double
    std::printf("%.1f\n", takes_double(f));          // 9. float to double
    std::printf("%.1f\n", takes_double(big));        // 10. long to double
    takes_float(d);                                  // 11. double to float
    takes_base_ref(derived);                         // 12. derived to base reference
    takes_base_value(derived);                       // 13. derived to base by value
    takes_base_pointer(&derived);                    // 14. derived pointer to base pointer
    takes_const_base_pointer(&derived);              // 15. derived pointer to const base pointer
    takes_pointer(array);                            // 16. array to pointer
    takes_pointer(nullptr);                          // 17. nullptr to pointer
    takes_void_pointer(ip);                          // 18. int pointer to void pointer
    takes_const_pointer(ip);                         // 19. int pointer to const int pointer
    takes_char_pointer(mutable_text);                // 20. char pointer to const char pointer
    return 0;
}
