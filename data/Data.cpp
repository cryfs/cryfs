#include "Data.h"
#include <stdexcept>

using std::istream;
using std::ofstream;
using std::ifstream;
using std::ios;

namespace bf = boost::filesystem;

namespace cpputils {

boost::optional<Data> Data::LoadFromFile(const bf::path &filepath) {
  ifstream file(filepath.c_str(), ios::binary);
  if (!file.good()) {
    return boost::none;
  }
  size_t size = _getStreamSize(file);

  Data result(size);
  result._readFromStream(file);
  return std::move(result);
}

size_t Data::_getStreamSize(istream &stream) {
  auto current_pos = stream.tellg();

  //Retrieve length
  stream.seekg(0, stream.end);
  auto endpos = stream.tellg();

  //Restore old position
  stream.seekg(current_pos, stream.beg);

  return endpos - current_pos;
}


void Data::_readFromStream(istream &stream) {
  stream.read((char*)_data, _size);
}

}
