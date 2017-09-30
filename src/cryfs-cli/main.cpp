#include "Cli.h"
#include <cpp-utils/random/Random.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include <cpp-utils/network/CurlHttpClient.h>
#include <cpp-utils/io/IOStreamConsole.h>

using namespace cryfs;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::CurlHttpClient;
using cpputils::IOStreamConsole;
using cpputils::make_unique_ref;
using std::make_shared;
using std::cerr;

int main(int argc, const char *argv[]) {
    try {
        auto &keyGenerator = Random::OSRandom();
        return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>())
            .main(argc, argv, make_unique_ref<CurlHttpClient>());
    } catch (const std::exception &e) {
        cerr << "Error: " << e.what();
        return EXIT_FAILURE;
    }
}
