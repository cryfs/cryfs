#include "Cli.h"
#include <cpp-utils/random/Random.h>
#include <cpp-utils/io/IOStreamConsole.h>
#include <cryfs/CryfsException.h>

#if defined(_MSC_VER)
#include <cpp-utils/network/WinHttpClient.h>
#else
#include <cpp-utils/network/CurlHttpClient.h>
#endif

using namespace cryfs;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::IOStreamConsole;
using cpputils::make_unique_ref;
using std::make_shared;
using std::cerr;

int main(int argc, const char *argv[]) {
    try {
        auto &keyGenerator = Random::OSRandom();
#if defined(_MSC_VER)
		    auto httpClient = make_unique_ref<cpputils::WinHttpClient>();
#else
		    auto httpClient = make_unique_ref<cpputils::CurlHttpClient>();
#endif
        return Cli(keyGenerator, SCrypt::DefaultSettings, make_shared<IOStreamConsole>())
            .main(argc, argv, std::move(httpClient));
    } catch (const CryfsException &e) {
        if (e.errorCode() != ErrorCode::Success) {
            std::cerr << "Error: " << e.what() << std::endl;
        }
        return exitCode(e.errorCode());
    } catch (const std::exception &e) {
        cerr << "Error: " << e.what();
        return exitCode(ErrorCode::UnspecifiedError);
    }
}
