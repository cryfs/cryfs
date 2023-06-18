#pragma once
#ifndef MESSMER_FSPP_TEST_TESTUTILS_INMEMORYFILE_H_
#define MESSMER_FSPP_TEST_TESTUTILS_INMEMORYFILE_H_

#include <cpp-utils/data/Data.h>
#include <fspp/fs_interface/Types.h>

class InMemoryFile {
public:
  InMemoryFile(cpputils::Data data);
  virtual ~InMemoryFile();

  fspp::num_bytes_t read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const;

  const void *data() const;
  fspp::num_bytes_t size() const;

  bool fileContentEquals(const cpputils::Data &expected, fspp::num_bytes_t offset) const;

protected:
  cpputils::Data _data;
};

class WriteableInMemoryFile: public InMemoryFile {
public:
  WriteableInMemoryFile(cpputils::Data data);

  void write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset);

  bool sizeUnchanged() const;
  bool regionUnchanged(fspp::num_bytes_t offset, fspp::num_bytes_t count) const;

private:
  void _extendFileSizeIfNecessary(fspp::num_bytes_t size);
  void _extendFileSize(fspp::num_bytes_t size);

  cpputils::Data _originalData;
};


#endif
