#include "CryConfigFile.h"
#include <fstream>
#include <boost/filesystem.hpp>

using boost::optional;
using boost::none;
using std::ifstream;
using std::ofstream;
namespace bf = boost::filesystem;

namespace cryfs {

CryConfigFile CryConfigFile::create(const bf::path &path, CryConfig config) {
    return CryConfigFile(path, std::move(config));
}

optional<CryConfigFile> CryConfigFile::load(const bf::path &path) {
    if (!bf::exists(path)) {
        return none;
    }
    ifstream file(path.native());
    CryConfig config;
    config.load(file);
    return CryConfigFile(path, std::move(config));
}

CryConfigFile::CryConfigFile(const bf::path &path, CryConfig config)
    : _path (path), _config(std::move(config)) {
}

CryConfigFile::CryConfigFile(CryConfigFile &&rhs)
    : _path(std::move(rhs._path)), _config(std::move(rhs._config)) {
}

void CryConfigFile::save() const {
    ofstream file(_path.native(), ofstream::out|ofstream::binary|ofstream::trunc);
    _config.save(file);
}

CryConfig *CryConfigFile::config() {
    return &_config;
}

}
