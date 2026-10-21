struct Other {
    int value;
};

int takes_int(int v) {
    return v;
}

int main() {
    Other other;
    return takes_int(other);
}
