struct P {
    int value;
    int* operator->() { return &value; }
};

int main() {
    P p{1};
    return p->value;
}
