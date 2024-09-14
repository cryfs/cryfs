#include "Cli.h"
#include <cpp-utils/random/Random.h>
#include <cpp-utils/io/IOStreamConsole.h>
#include <cpp-utils/network/CurlHttpClient.h>
#include <cryfs/impl/CryfsException.h>

#if defined(_MSC_VER)
#include <VersionHelpers.h>
#endif

using namespace cryfs_cli;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::IOStreamConsole;
#ifdef CRYFS_UPDATE_CHECKS
using cpputils::make_unique_ref;
#endif
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
        auto *keyGenerator = Random::OSRandom();

        return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>())
            .main(
                argc,
                argv,
                #ifdef CRYFS_UPDATE_CHECKS
                make_unique_ref<cpputils::CurlHttpClient>(),
                #endif
                []{}
            );
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
