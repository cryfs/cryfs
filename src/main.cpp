#include "cli/Cli.h"
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/cpp-utils/crypto/kdf/Scrypt.h>

using namespace cryfs;
using cpputils::Random;
using cpputils::SCrypt;

int main(int argc, char *argv[]) {
    auto &keyGenerator = Random::OSRandom();
    return Cli(keyGenerator, SCrypt::DefaultSettings).main(argc, argv);
}
