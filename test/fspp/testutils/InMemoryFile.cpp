#include "InMemoryFile.h"

using cpputils::Data;

InMemoryFile::InMemoryFile(Data data): _data(std::move(data)) {
}

InMemoryFile::~InMemoryFile() {
}

fspp::num_bytes_t InMemoryFile::read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const {
  fspp::num_bytes_t realCount = std::min(count, fspp::num_bytes_t(_data.size()) - offset);
  std::memcpy(buf, _data.dataOffset(offset.value()), realCount.value());
  return realCount;
}

const void *InMemoryFile::data() const {
  return _data.data();
}

bool InMemoryFile::fileContentEquals(const Data &expected, fspp::num_bytes_t offset) const {
  return 0 == std::memcmp(expected.data(), _data.dataOffset(offset.value()), expected.size());
}

fspp::num_bytes_t InMemoryFile::size() const {
  return fspp::num_bytes_t(_data.size());
}

WriteableInMemoryFile::WriteableInMemoryFile(Data data): InMemoryFile(std::move(data)), _originalData(_data.copy()) {
}

void WriteableInMemoryFile::write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  _extendFileSizeIfNecessary(count + offset);

  std::memcpy(_data.dataOffset(offset.value()), buf, count.value());
}

void WriteableInMemoryFile::_extendFileSizeIfNecessary(fspp::num_bytes_t size) {
  if (size > fspp::num_bytes_t(_data.size())) {
    _extendFileSize(size);
  }
}

void WriteableInMemoryFile::_extendFileSize(fspp::num_bytes_t size) {
  Data newfile(size.value());
  std::memcpy(newfile.data(), _data.data(), _data.size());
  _data = std::move(newfile);
}

bool WriteableInMemoryFile::sizeUnchanged() const {
  return _data.size() == _originalData.size();
}

bool WriteableInMemoryFile::regionUnchanged(fspp::num_bytes_t offset, fspp::num_bytes_t count) const {
  return 0 == std::memcmp(_data.dataOffset(offset.value()), _originalData.dataOffset(offset.value()), count.value());
}
