#include "VirtualTestFile.h"

#include <algorithm>
#include <cstring>

using std::min;

VirtualTestFile::VirtualTestFile(size_t size): _fileData(new char[size]), _size(size) {
  fillFileWithRandomData();
}

VirtualTestFile::~VirtualTestFile() {
  delete[] _fileData;
}

void VirtualTestFile::fillFileWithRandomData() {
  long long int val = 1;
  for(size_t i=0; i<_size/sizeof(long long int); ++i) {
    //MMIX linear congruential generator
    val *= 6364136223846793005L;
    val += 1442695040888963407;
    reinterpret_cast<long long int*>(_fileData)[i] = val;
  }
}

int VirtualTestFile::read(void *buf, size_t count, off_t offset) {
  size_t realCount = min(count, _size - offset);
  memcpy(buf, _fileData+offset, realCount);
  return realCount;
}

bool VirtualTestFile::fileContentEqual(char *content, size_t count, off_t offset) {
  return 0 == memcmp(content, _fileData + offset, count);
}

VirtualTestFileWriteable::VirtualTestFileWriteable(size_t size)
  :VirtualTestFile(size) {
  originalFileData = new char[size];
  memcpy(originalFileData, _fileData, size);
}

VirtualTestFileWriteable::~VirtualTestFileWriteable() {
  delete[] originalFileData;
}
