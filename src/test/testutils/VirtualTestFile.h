#pragma once
#ifndef TEST_TESTUTILS_VIRTUALTESTFILE_H_
#define TEST_TESTUTILS_VIRTUALTESTFILE_H_

#include <cstdio>

class VirtualTestFile {
public:
  VirtualTestFile(size_t size);
  virtual ~VirtualTestFile();

  int read(void *buf, size_t count, off_t offset);

  // Return true, iff the given data is equal to the data of the file at the given offset.
  bool fileContentEqual(char *content, size_t count, off_t offset);

protected:
  char *_fileData;
  size_t _size;

private:
  void fillFileWithRandomData();
};

class VirtualTestFileWriteable: public VirtualTestFile {
public:
  VirtualTestFileWriteable(size_t size);
  virtual ~VirtualTestFileWriteable();
private:
  char *originalFileData;
};

#endif
