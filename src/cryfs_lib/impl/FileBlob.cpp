#include <cryfs_lib/impl/FileBlob.h>

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
  *magicNumber() = MagicNumbers::FILE;
}

unsigned char *FileBlob::magicNumber() {
  return const_cast<unsigned char*>(magicNumber(const_cast<const Blob&>(*_blob)));
}

const unsigned char *FileBlob::magicNumber(const blobstore::Blob &blob) {
  return (unsigned char*)blob.data();
}

bool FileBlob::IsFile(const Blob &blob) {
  return *magicNumber(blob) == MagicNumbers::FILE;
}

}
