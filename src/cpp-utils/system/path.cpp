#include <cpp-utils/system/path.h>
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;

namespace cpputils {

bf::path find_longest_existing_path_prefix(const bf::path& path) {
    if (path.size() == 0) {
        return path;
    }
    bf::path result;
    for(const auto& component : path) {
        if (bf::exists(result / component)) {
            result /= component;
        } else {
            break;
        }
    }
    return result;
}

}
