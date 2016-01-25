#include "cli/Cli.h"
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/cpp-utils/crypto/kdf/Scrypt.h>

using namespace cryfs;
using cpputils::Random;
using cpputils::SCrypt;
using std::make_shared;
using cpputils::IOStreamConsole;

int main(int argc, char *argv[]) {
    auto &keyGenerator = Random::OSRandom();
    return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>()).main(argc, argv);
}
