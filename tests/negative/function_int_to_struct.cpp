struct Other {
    int value;
};

void takes_struct(Other other) {
    other.value = 1;
}

int main() {
    takes_struct(5);
    return 0;
}
