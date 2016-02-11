#pragma once
#ifndef MESSMER_FSPP_TEST_TESTUTILS_INMEMORYFILE_H_
#define MESSMER_FSPP_TEST_TESTUTILS_INMEMORYFILE_H_

#include <cpp-utils/data/Data.h>

class InMemoryFile {
public:
  InMemoryFile(cpputils::Data data);
  virtual ~InMemoryFile();

  int read(void *buf, size_t count, off_t offset) const;

  const void *data() const;
  size_t size() const;

  bool fileContentEquals(const cpputils::Data &expected, off_t offset) const;

protected:
  cpputils::Data _data;
};

class WriteableInMemoryFile: public InMemoryFile {
public:
  WriteableInMemoryFile(cpputils::Data data);

  void write(const void *buf, size_t count, off_t offset);

  bool sizeUnchanged() const;
  bool regionUnchanged(off_t offset, size_t count) const;

private:
  void _extendFileSizeIfNecessary(size_t size);
  void _extendFileSize(size_t size);

  cpputils::Data _originalData;
};


#endif
