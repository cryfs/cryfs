#include "cli/Cli.h"
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/cpp-utils/crypto/kdf/Scrypt.h>
#include <messmer/cpp-utils/network/CurlHttpClient.h>

using namespace cryfs;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::CurlHttpClient;
using std::make_shared;
using cpputils::IOStreamConsole;

int main(int argc, char *argv[]) {
    auto &keyGenerator = Random::OSRandom();
    return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>(), make_shared<CurlHttpClient>()).main(argc, argv);
}
