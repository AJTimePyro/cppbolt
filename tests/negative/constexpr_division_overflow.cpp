// Negative test: integer overflow in constant expression division
int arr[((-1LL << 63) * (1LL << 64)) / -1];

int main() {
    return 0;
}
