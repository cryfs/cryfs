#include "Data.h"
#include <stdexcept>
#include <vendor_cryptopp/hex.h>
#include <cpp-utils/crypto/cryptopp_byte.h>

using std::istream;
using std::ofstream;
using std::ifstream;
using std::ios;
using boost::optional;

namespace bf = boost::filesystem;

namespace cpputils {

optional<Data> Data::LoadFromFile(const bf::path &filepath) {
  ifstream file(filepath.string().c_str(), ios::binary);
  if (!file.good()) {
    return boost::none;
  }
  optional<Data> result(LoadFromStream(file));
  if (!file.good()) {
    throw std::runtime_error("Error reading from file");
  }
  return result;
}

std::streampos Data::_getStreamSize(istream &stream) {
  auto current_pos = stream.tellg();

  //Retrieve length
  stream.seekg(0, stream.end);
  auto endpos = stream.tellg();

  //Restore old position
  stream.seekg(current_pos, stream.beg);

  return endpos - current_pos;
}

Data Data::LoadFromStream(istream &stream, size_t size) {
  Data result(size);
  stream.read(static_cast<char*>(result.data()), result.size());
  return result;
}

Data Data::FromString(const std::string &data, unique_ref<Allocator> allocator) {
  ASSERT(data.size() % 2 == 0, "hex encoded data cannot have odd number of characters");
  Data result(data.size() / 2, std::move(allocator));
  {
    CryptoPP::StringSource _1(data, true,
      new CryptoPP::HexDecoder(
        new CryptoPP::ArraySink(static_cast<CryptoPP::byte*>(result._data), result.size())
      )
    );
  }
  return result;
}

std::string Data::ToString() const {
  std::string result;
  {
    CryptoPP::ArraySource _1(static_cast<const CryptoPP::byte*>(_data), _size, true,
      new CryptoPP::HexEncoder(
          new CryptoPP::StringSink(result)
      )
    );
  }
  ASSERT(result.size() == 2 * _size, "Created wrongly sized string");
  return result;
}

}
