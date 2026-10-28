#include <stdio.h>

struct Counter {
    int count;
    Counter(int c) : count(c) {}

    // Prefix increment: returns reference to self
    Counter& operator++() {
        count += 1;
        return *this;
    }

    // Postfix increment: takes dummy int, returns copy of old value
    Counter operator++(int) {
        Counter temp = *this;
        count += 1;
        return temp;
    }

    // Postfix decrement: takes dummy int
    Counter operator--(int) {
        Counter temp = *this;
        count -= 1;
        return temp;
    }
};

int main() {
    Counter c(5);
    Counter old = c++;
    printf("%d\n", old.count); // 5
    printf("%d\n", c.count);   // 6

    Counter old2 = c--;
    printf("%d\n", old2.count); // 6
    printf("%d\n", c.count);    // 5

    ++c;
    printf("%d\n", c.count);    // 6
    return 0;
}
