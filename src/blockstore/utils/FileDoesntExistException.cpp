#include "FileDoesntExistException.h"

namespace bf = boost::filesystem;

using std::string;

namespace blockstore {

FileDoesntExistException::FileDoesntExistException(const bf::path &filepath)
: runtime_error(string("The file ")+filepath.string()+" doesn't exist") {
}

FileDoesntExistException::~FileDoesntExistException() {
}

}
