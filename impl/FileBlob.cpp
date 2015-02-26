#include "FileBlob.h"

#include "MagicNumbers.h"

using std::unique_ptr;
using blobstore::Blob;

namespace cryfs {

FileBlob::FileBlob(unique_ptr<Blob> blob)
: _blob(std::move(blob)) {
}

FileBlob::~FileBlob() {
}

void FileBlob::InitializeEmptyFile() {
  _blob->resize(1);
  unsigned char magicNumber = MagicNumbers::FILE;
  _blob->write(&magicNumber, 0, 1);
}

unsigned char FileBlob::magicNumber() const {
  return magicNumber(*_blob);
}

unsigned char FileBlob::magicNumber(const blobstore::Blob &blob) {
  unsigned char value;
  blob.read(&value, 0, 1);
  return value;
}

bool FileBlob::IsFile(const Blob &blob) {
  return magicNumber(blob) == MagicNumbers::FILE;
}

}
