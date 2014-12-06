#include <blobstore/implementations/ondisk/RandomKeyGenerator.h>

using std::string;

#include <crypto++/hex.h>
#include <crypto++/osrng.h>

using CryptoPP::AutoSeededRandomPool;
using CryptoPP::ArraySource;
using CryptoPP::StringSink;
using CryptoPP::HexEncoder;

using std::make_unique;

namespace blobstore {
namespace ondisk {

constexpr unsigned int RandomKeyGenerator::KEYLENGTH_ENTROPY;
constexpr unsigned int RandomKeyGenerator::KEYLENGTH;

namespace {
string encodeKeyToHex(const byte *data);
}

RandomKeyGenerator::RandomKeyGenerator()
: _randomPool(make_unique<AutoSeededRandomPool>()) {
}

RandomKeyGenerator::~RandomKeyGenerator() {
}

RandomKeyGenerator &RandomKeyGenerator::singleton() {
  static RandomKeyGenerator singleton;
  return singleton;
}

string RandomKeyGenerator::create() {
  byte key[KEYLENGTH_ENTROPY];
  _randomPool->GenerateBlock(key, KEYLENGTH_ENTROPY);
  return encodeKeyToHex(key);
}

namespace {
string encodeKeyToHex(const byte *data) {
  string result;
  ArraySource(data, RandomKeyGenerator::KEYLENGTH_ENTROPY, true,
      new HexEncoder(new StringSink(result))
  );
  assert(result.size() == RandomKeyGenerator::KEYLENGTH);
  return result;
}
}

} /* namespace ondisk */
} /* namespace blobstore */
