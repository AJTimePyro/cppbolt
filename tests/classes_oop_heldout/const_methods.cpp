#include <cstdio>

class SensorConfig {
    int threshold;
    int sample_rate;
public:
    SensorConfig(int t, int s) : threshold(t), sample_rate(s) {}
    int get_threshold() const {
        return threshold;
    }
    int get_sample_rate() const {
        return sample_rate;
    }
};

void report_metrics(const SensorConfig& cfg) {
    std::printf("metrics: th=%d, rate=%d\n", cfg.get_threshold(), cfg.get_sample_rate());
}

int main() {
    SensorConfig sc(50, 1000);
    report_metrics(sc);
    return 0;
}
