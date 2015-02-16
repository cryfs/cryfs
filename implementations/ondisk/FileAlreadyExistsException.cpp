#include <messmer/blockstore/implementations/ondisk/FileAlreadyExistsException.h>

namespace bf = boost::filesystem;

using std::runtime_error;
using std::string;

namespace blockstore {
namespace ondisk {

FileAlreadyExistsException::FileAlreadyExistsException(const bf::path &filepath)
: runtime_error(string("The file ")+filepath.c_str()+" already exists") {
}

FileAlreadyExistsException::~FileAlreadyExistsException() {
}

}
}
