#include "MyClientId.h"
#include <fstream>
#include <cpp-utils/random/Random.h>
#include <blockstore/implementations/integrity/KnownBlockVersions.h>

using boost::optional;
using boost::none;
using std::ifstream;
using std::ofstream;
using cpputils::Random;
using blockstore::integrity::KnownBlockVersions;
namespace bf = boost::filesystem;

namespace cryfs {

    MyClientId::MyClientId(const bf::path &statePath)
            :_stateFilePath(statePath / "myClientId") {
    }

    uint32_t MyClientId::loadOrGenerate() const {
        auto loaded = _load();
        if (loaded != none) {
            return *loaded;
        }
        // If it couldn't be loaded, generate a new client id.
        auto generated = _generate();
        _save(generated);
        return generated;
    }

    uint32_t MyClientId::_generate() {
        uint32_t result;
        do {
            result = *reinterpret_cast<uint32_t*>(Random::PseudoRandom().getFixedSize<sizeof(uint32_t)>().data());
        } while(result == KnownBlockVersions::CLIENT_ID_FOR_DELETED_BLOCK); // Safety check - CLIENT_ID_FOR_DELETED_BLOCK shouldn't be used by any valid client.
        return result;
    }

    optional<uint32_t> MyClientId::_load() const {
        ifstream file(_stateFilePath.native());
        if (!file.good()) {
            // State file doesn't exist
            return none;
        }
        uint32_t value;
        file >> value;
        return value;
    }

    void MyClientId::_save(uint32_t clientId) const {
        ofstream file(_stateFilePath.native());
        file << clientId;
    }
}
