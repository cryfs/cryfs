#include "FileBlob.h"

#include "MagicNumbers.h"
#include <messmer/blockstore/utils/Key.h>
#include <cassert>

using blobstore::Blob;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

namespace cryfs {

FileBlob::FileBlob(unique_ref<Blob> blob)
: _blob(std::move(blob)) {
}

FileBlob::~FileBlob() {
}

unique_ref<FileBlob> FileBlob::InitializeEmptyFile(unique_ref<Blob> blob) {
  blob->resize(1);
  unsigned char magicNumber = MagicNumbers::FILE;
  blob->write(&magicNumber, 0, 1);
  return make_unique_ref<FileBlob>(std::move(blob));
}

unsigned char FileBlob::magicNumber() const {
  unsigned char value;
  _blob->read(&value, 0, 1);
  return value;
}

ssize_t FileBlob::read(void *target, uint64_t offset, uint64_t count) const {
  return _blob->tryRead(target, offset + 1, count);
}

void FileBlob::write(const void *source, uint64_t offset, uint64_t count) {
  _blob->write(source, offset + 1, count);
}

void FileBlob::flush() {
  _blob->flush();
}

blockstore::Key FileBlob::key() const {
  	return _blob->key();
}

void FileBlob::resize(off_t size) {
  _blob->resize(size+1);
}

off_t FileBlob::size() const {
  return _blob->size()-1;
}

}
