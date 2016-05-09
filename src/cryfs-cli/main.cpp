#include "Cli.h"
#include <cpp-utils/random/Random.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include <cpp-utils/network/CurlHttpClient.h>

using namespace cryfs;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::CurlHttpClient;
using std::make_shared;
using cpputils::IOStreamConsole;

int main(int argc, const char *argv[]) {
    auto &keyGenerator = Random::OSRandom();
    return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>(), make_shared<CurlHttpClient>()).main(argc, argv);
}
