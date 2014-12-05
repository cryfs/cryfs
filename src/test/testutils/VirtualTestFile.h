#pragma once
#ifndef TEST_TESTUTILS_VIRTUALTESTFILE_H_
#define TEST_TESTUTILS_VIRTUALTESTFILE_H_

#include <cstdio>

class VirtualTestFile {
public:
  VirtualTestFile(size_t size, long long int IV = 1);
  virtual ~VirtualTestFile();

  int read(void *buf, size_t count, off_t offset);

  // Return true, iff the given data is equal to the data of the file at the given offset.
  bool fileContentEqual(const char *content, size_t count, off_t offset);

  const char *data() const;

  size_t size() const;

protected:
  char *_fileData;
  size_t _size;

private:
  void fillFileWithRandomData(long long int IV);
};

class VirtualTestFileWriteable: public VirtualTestFile {
public:
  VirtualTestFileWriteable(size_t size, long long int IV = 1);
  virtual ~VirtualTestFileWriteable();

  void write(const void *buf, size_t count, off_t offset);

  bool sizeUnchanged();
  bool regionUnchanged(off_t offset, size_t count);

private:
  void extendFileSizeIfNecessary(size_t size);
  void extendFileSize(size_t size);

  char *_originalFileData;
  size_t _originalSize;
};

#endif
