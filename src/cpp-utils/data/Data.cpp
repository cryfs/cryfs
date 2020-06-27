#include "Data.h"
#include <stdexcept>
#include <boost/iostreams/device/file.hpp>
#include <boost/iostreams/device/file_descriptor.hpp>
#include <boost/iostreams/stream.hpp>
#include <vendor_cryptopp/hex.h>

using std::istream;
using std::ofstream;
using std::ifstream;
using std::ios;
using boost::optional;

namespace bf = boost::filesystem;
namespace bio = boost::iostreams;

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

namespace {
  void _fsync(bio::file_descriptor_sink::handle_type file_handle) {
    #if defined(_MSC_VER)
      int status = ::fflush(file_handle);
      if (0 != status) {
        throw std::runtime_error("Error in Data::StoreToFile: fflush failed. Errno: " + std::to_string(errno));
      }
    #elif defined(F_FULLFSYNC)
      // On osx, if F_FULLFSYNC is defined, then fsync can't be relied on and we need to use fcntl instead.
      // See https://www.slideshare.net/nan1nan1/eat-my-data
      int status = ::fcntl(file_handle, F_FULLFSYNC, nullptr);
      if (0 != status) {
        status = ::fsync(file_handle);
        if (0 != status) {
          throw std::runtime_error("Error in Data::StoreToFile: fsync failed. Errno: " + std::to_string(errno));
        }
      }
    #else
      int status = ::fsync(file_handle);
      if (0 != status) {
        throw std::runtime_error("Error in Data::StoreToFile: fsync failed. Errno: " + std::to_string(errno));
      }
    #endif
  }

  void _rename(const bf::path& source, const bf::path& target) {
    #if defined(_MSC_VER)
      BOOL success = ReplaceFileA(target.string().c_str(), source.string().c_str(), nullptr, nullptr, nullptr)
      if (!success) {
        throw std::runtime_error("Error in Data::StoreToFile: ReplaceFileA failed. Code: " + success + ". Error: " + GetLastError());
      }
    #else
      int status = ::rename(source.string().c_str(), target.string().c_str());
      if (0 != status) {
        throw std::runtime_error("Error in Data::StoreToFile: rename failed. Errno: " + std::to_string(errno));
      }
    #endif
  }
}

void Data::StoreToFile(const bf::path &filepath) const {
  // Atomic file write strategy:
  // 1. Write to a temporary file
  // 2. Fsync any changes
  // 3. Rename the temporary file to the file we actually wanted to write to

  bf::path temp_path = filepath.string() + ".tmp";
  bio::file_descriptor_sink file(temp_path, std::ios::binary | std::ios::trunc);
  if (!file.is_open()) {
    throw std::runtime_error("Error in Data::StoreToFile: opening file descriptor failed");
  }
  bio::stream<bio::file_descriptor_sink> stream(file);
  if (!stream.good()) {
    throw std::runtime_error("Error in Data::StoreToFile: stream creation failed");
  }
  StoreToStream(stream);
  if (!stream.good()) {
    throw std::runtime_error("Error in Data::StoreToFile: write failed");
  }
  // TODO Windows: https://stackoverflow.com/questions/32575244/using-fsync-for-saving-binary-files-in-c-or-c
  stream.flush();
  if (!stream.good()) {
    throw std::runtime_error("Error in Data::StoreToFile: flush failed");
  }
  _fsync(file.handle());
  stream.close();
  _rename(temp_path, filepath);
}

}
