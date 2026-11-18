int main() {
    {
        using Local = int;
        Local x = 10;
    }
    Local y = 20;
    return 0;
}
