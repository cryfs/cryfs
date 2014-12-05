#include "VirtualTestFile.h"

#include <algorithm>
#include <cstring>

using std::min;

VirtualTestFile::VirtualTestFile(size_t size, long long int IV): _fileData(new char[size]), _size(size) {
  fillFileWithRandomData(IV);
}

VirtualTestFile::~VirtualTestFile() {
  delete[] _fileData;
}

void VirtualTestFile::fillFileWithRandomData(long long int IV) {
  long long int val = IV;
  for(size_t i=0; i<_size/sizeof(long long int); ++i) {
    //MMIX linear congruential generator
    val *= 6364136223846793005L;
    val += 1442695040888963407;
    reinterpret_cast<long long int*>(_fileData)[i] = val;
  }
}

const char *VirtualTestFile::data() const {
  return _fileData;
}

int VirtualTestFile::read(void *buf, size_t count, off_t offset) {
  size_t realCount = min(count, _size - offset);
  memcpy(buf, _fileData+offset, realCount);
  return realCount;
}

size_t VirtualTestFile::size() const {
  return _size;
}

bool VirtualTestFile::fileContentEqual(const char *content, size_t count, off_t offset) {
  return 0 == memcmp(content, _fileData + offset, count);
}

VirtualTestFileWriteable::VirtualTestFileWriteable(size_t size, long long int IV)
  :VirtualTestFile(size, IV), _originalSize(size) {
  _originalFileData = new char[size];
  memcpy(_originalFileData, _fileData, size);
}

VirtualTestFileWriteable::~VirtualTestFileWriteable() {
  delete[] _originalFileData;
}

void VirtualTestFileWriteable::write(const void *buf, size_t count, off_t offset) {
  extendFileSizeIfNecessary(count + offset);

  memcpy(_fileData+offset, buf, count);
}

void VirtualTestFileWriteable::extendFileSizeIfNecessary(size_t size) {
  if (size > _size) {
    extendFileSize(size);
  }
}

void VirtualTestFileWriteable::extendFileSize(size_t size) {
  char *newfile = new char[size];
  memcpy(newfile, _fileData, _size);
  delete[] _fileData;
  _fileData = newfile;
  _size = size;
}

bool VirtualTestFileWriteable::sizeUnchanged() {
  return _size == _originalSize;
}

bool VirtualTestFileWriteable::regionUnchanged(off_t offset, size_t count) {
  return 0 == memcmp(_fileData+offset, _originalFileData+offset, count);
}
