#include "Cli.h"
#include <messmer/cpp-utils/random/Random.h>

using namespace cryfs;

int main(int argc, char *argv[]) {
    auto &keyGenerator = cpputils::Random::OSRandom();
    return Cli(keyGenerator).main(argc, argv);
}
