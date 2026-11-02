// café 🦀: declarations and lambda scopes must retain canonical UTF-8 bytes.
namespace Boot {
class Kernel {
public:
    void start(int cpu);
    void start(bool cold);
};
void Kernel::start(int cpu) {
    auto runner = [cpu] {
        return cpu + 1;
    };
    runner();
}
void Kernel::start(bool cold) {
    (void)cold;
}
}
int main() {
    return 0;
}
