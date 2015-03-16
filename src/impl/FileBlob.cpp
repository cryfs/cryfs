#include "FileBlob.h"

#include "MagicNumbers.h"
#include <messmer/blockstore/utils/Key.h>

using std::unique_ptr;
using std::make_unique;
using blobstore::Blob;

namespace cryfs {

FileBlob::FileBlob(unique_ptr<Blob> blob)
: _blob(std::move(blob)) {
}

FileBlob::~FileBlob() {
}

unique_ptr<FileBlob> FileBlob::InitializeEmptyFile(unique_ptr<Blob> blob) {
  blob->resize(1);
  unsigned char magicNumber = MagicNumbers::FILE;
  blob->write(&magicNumber, 0, 1);
  return make_unique<FileBlob>(std::move(blob));
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
