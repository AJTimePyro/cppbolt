struct P {
    int value;
    int operator*() const { return value; }
};

int main() {
    P p{1};
    return *p;
}
