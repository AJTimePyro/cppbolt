#include <cstdio>

class FileStream {
public:
    FileStream() { std::printf("FileStream ctor\n"); }
    virtual ~FileStream() { std::printf("FileStream dtor\n"); }
};

class EncryptedFileStream : public FileStream {
    int* key_data;
public:
    EncryptedFileStream() : key_data(new int(9999)) {
        std::printf("EncryptedFileStream ctor\n");
    }
    virtual ~EncryptedFileStream() {
        std::printf("EncryptedFileStream dtor\n");
        delete key_data;
    }
};

int main() {
    FileStream* fs = new EncryptedFileStream();
    delete fs;
    return 0;
}
