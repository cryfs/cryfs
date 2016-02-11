#include "FileDoesntExistException.h"

namespace bf = boost::filesystem;

using std::runtime_error;
using std::string;

namespace blockstore {

FileDoesntExistException::FileDoesntExistException(const bf::path &filepath)
: runtime_error(string("The file ")+filepath.c_str()+" doesn't exist") {
}

FileDoesntExistException::~FileDoesntExistException() {
}

}
