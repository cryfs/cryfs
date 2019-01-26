#include "Cli.h"
#include <cpp-utils/random/Random.h>
#include <cpp-utils/io/IOStreamConsole.h>
#include <cryfs/impl/CryfsException.h>

#if defined(_MSC_VER)
#include <cpp-utils/network/WinHttpClient.h>
#include <VersionHelpers.h>
#else
#include <cpp-utils/network/CurlHttpClient.h>
#endif

using namespace cryfs_cli;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::IOStreamConsole;
using cpputils::make_unique_ref;
using std::make_shared;
using std::cerr;

int main(int argc, const char *argv[]) {
#if defined(_MSC_VER)
    if (!IsWindows7SP1OrGreater()) {
       std::cerr << "CryFS is currently only supported on Windows 7 SP1 (or later)." << std::endl;
       exit(1);
    }
#endif

    try {
        auto &keyGenerator = Random::OSRandom();
#if defined(_MSC_VER)
        auto httpClient = make_unique_ref<cpputils::WinHttpClient>();
#else
        auto httpClient = make_unique_ref<cpputils::CurlHttpClient>();
#endif
        return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>())
            .main(argc, argv, std::move(httpClient), []{});
    } catch (const cryfs::CryfsException &e) {
        if (e.what() != string()) {
            std::cerr << "Error: " << e.what() << std::endl;
        }
        return exitCode(e.errorCode());
    } catch (const std::exception &e) {
        cerr << "Error: " << e.what();
        return exitCode(cryfs::ErrorCode::UnspecifiedError);
    }
}
