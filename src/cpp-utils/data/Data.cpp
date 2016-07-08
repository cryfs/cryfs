#include "Data.h"
#include <stdexcept>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <fspp/impl/Profiler.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

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
  auto result = LoadFromStream(file);
  if (!file.good()) {
    throw std::runtime_error("Error reading from file");
  }
  return result;
}


    void Data::StoreToFile(const boost::filesystem::path &filepath) const {
      fspp::Profiler p(&blockstore::ondisk::OnDiskBlockStore::loadFromDiskProfile7);
      int fd = ::open(filepath.native().c_str(), O_WRONLY); // TODO O_TRUNC?
      if (-1 == fd) {
          throw std::runtime_error("Opening file for block failed. Errno: " + std::to_string(errno));
      }
      ssize_t written = ::write(fd, _data, _size);
      if (written != (ssize_t)_size) { // TODO Which way cast?
          throw std::runtime_error("Writing to opened block failed. Errno: " + std::to_string(errno));
      }
      // TODO Retry if not fully written?
      if (0 != ::close(fd)) {
          throw std::runtime_error("Failed closing opened block. Errno: " + std::to_string(errno));
      }
    }

    void Data::StoreToNewFile(const boost::filesystem::path &filepath) const {
        fspp::Profiler p(&blockstore::ondisk::OnDiskBlockStore::loadFromDiskProfile7);
        int fd = ::open(filepath.native().c_str(), O_CREAT | O_EXCL | O_WRONLY, S_IRUSR | S_IWUSR);
        if (-1 == fd) {
            throw std::runtime_error("Creating file for block failed. Errno: " + std::to_string(errno));
        }
        ssize_t written = ::write(fd, _data, _size);
        if (written != (ssize_t)_size) { // TODO Which way cast?
            throw std::runtime_error("Writing to created block failed. Errno: " + std::to_string(errno));
        }
        // TODO Retry if not fully written?
        if (0 != ::close(fd)) {
            throw std::runtime_error("Failed closing created block. Errno: " + std::to_string(errno));
        }
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
  return std::move(result);
}

}
