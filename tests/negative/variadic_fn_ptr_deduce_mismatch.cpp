int one(int);

template <typename R, typename First, typename Second, typename... Rest>
void f(R (*fn)(First, Second, Rest...));

int main() {
    f(one);
    return 0;
}
