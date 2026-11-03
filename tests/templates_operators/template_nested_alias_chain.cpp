#include <cstdio>

template <typename T>
struct Pipeline {
    using Step1 = T;
    using Step2 = Step1;
    using Step3 = Step2;
    using Step4 = Step3;
    using Step5 = Step4;

    struct Processor {
        Step5 data;
        Processor(Step5 d) : data(d) {}
    };

    Processor proc;
    Pipeline(T val) : proc(val) {}
};

int main() {
    Pipeline<int> p(123);
    typename Pipeline<int>::Processor proc = p.proc;
    std::printf("val: %d\n", proc.data);
    return 0;
}
