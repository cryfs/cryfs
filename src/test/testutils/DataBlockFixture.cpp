#include <test/testutils/DataBlockFixture.h>
#include <algorithm>
#include <cstring>

using std::min;

DataBlockFixture::DataBlockFixture(size_t size, long long int IV): _fileData(new char[size]), _size(size) {
  fillFileWithRandomData(IV);
}

DataBlockFixture::~DataBlockFixture() {
  delete[] _fileData;
}

void DataBlockFixture::fillFileWithRandomData(long long int IV) {
  long long int val = IV;
  for(size_t i=0; i<_size/sizeof(long long int); ++i) {
    //MMIX linear congruential generator
    val *= 6364136223846793005L;
    val += 1442695040888963407;
    reinterpret_cast<long long int*>(_fileData)[i] = val;
  }
}

const char *DataBlockFixture::data() const {
  return _fileData;
}

int DataBlockFixture::read(void *buf, size_t count, off_t offset) {
  size_t realCount = min(count, _size - offset);
  memcpy(buf, _fileData+offset, realCount);
  return realCount;
}

size_t DataBlockFixture::size() const {
  return _size;
}

bool DataBlockFixture::fileContentEqual(const char *content, size_t count, off_t offset) {
  return 0 == memcmp(content, _fileData + offset, count);
}

DataBlockFixtureWriteable::DataBlockFixtureWriteable(size_t size, long long int IV)
  :DataBlockFixture(size, IV), _originalSize(size) {
  _originalFileData = new char[size];
  memcpy(_originalFileData, _fileData, size);
}

DataBlockFixtureWriteable::~DataBlockFixtureWriteable() {
  delete[] _originalFileData;
}

void DataBlockFixtureWriteable::write(const void *buf, size_t count, off_t offset) {
  extendFileSizeIfNecessary(count + offset);

  memcpy(_fileData+offset, buf, count);
}

void DataBlockFixtureWriteable::extendFileSizeIfNecessary(size_t size) {
  if (size > _size) {
    extendFileSize(size);
  }
}

void DataBlockFixtureWriteable::extendFileSize(size_t size) {
  char *newfile = new char[size];
  memcpy(newfile, _fileData, _size);
  delete[] _fileData;
  _fileData = newfile;
  _size = size;
}

bool DataBlockFixtureWriteable::sizeUnchanged() {
  return _size == _originalSize;
}

bool DataBlockFixtureWriteable::regionUnchanged(off_t offset, size_t count) {
  return 0 == memcmp(_fileData+offset, _originalFileData+offset, count);
}
