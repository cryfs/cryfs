#include "InMemoryFile.h"

using cpputils::Data;

InMemoryFile::InMemoryFile(Data data): _data(std::move(data)) {
}

InMemoryFile::~InMemoryFile() {
}

int InMemoryFile::read(void *buf, size_t count, off_t offset) const {
  size_t realCount = std::min(count, (size_t)(_data.size() - offset));
  std::memcpy(buf, (uint8_t*)_data.data() + offset, realCount);
  return realCount;
}

const void *InMemoryFile::data() const {
  return _data.data();
}

bool InMemoryFile::fileContentEquals(const Data &expected, off_t offset) const {
  return 0 == std::memcmp((uint8_t*)expected.data(), (uint8_t*)_data.data() + offset, expected.size());
}

size_t InMemoryFile::size() const {
  return _data.size();
}

WriteableInMemoryFile::WriteableInMemoryFile(Data data): InMemoryFile(std::move(data)), _originalData(_data.copy()) {
}

void WriteableInMemoryFile::write(const void *buf, size_t count, off_t offset) {
  _extendFileSizeIfNecessary(count + offset);

  std::memcpy((uint8_t*)_data.data() + offset, buf, count);
}

void WriteableInMemoryFile::_extendFileSizeIfNecessary(size_t size) {
  if (size > _data.size()) {
    _extendFileSize(size);
  }
}

void WriteableInMemoryFile::_extendFileSize(size_t size) {
  Data newfile(size);
  std::memcpy(newfile.data(), _data.data(), _data.size());
  _data = std::move(newfile);
}

bool WriteableInMemoryFile::sizeUnchanged() const {
  return _data.size() == _originalData.size();
}

bool WriteableInMemoryFile::regionUnchanged(off_t offset, size_t count) const {
  return 0 == std::memcmp((uint8_t*)_data.data() + offset, (uint8_t*)_originalData.data() + offset, count);
}
