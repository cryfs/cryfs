#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_CIPHER_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_CIPHER_H_

#include <boost/concept_check.hpp>
#include <cstdint>
#include "../../data/Data.h"
#include "../../random/Random.h"

using std::string;

namespace cpputils {

template<class X>
struct CipherConcept {
public:
  BOOST_CONCEPT_USAGE(CipherConcept) {
    same_type(UINT32_C(0), X::ciphertextSize(UINT32_C(5)));
    same_type(UINT32_C(0), X::plaintextSize(UINT32_C(5)));
    same_type(UINT32_C(0), X::KEYSIZE);
    same_type(UINT32_C(0), X::STRING_KEYSIZE);
    typename X::EncryptionKey key = X::EncryptionKey::CreateKey(Random::OSRandom(), X::KEYSIZE);
    same_type(Data(0), X::encrypt(static_cast<uint8_t*>(nullptr), UINT32_C(0), key));
    same_type(boost::optional<Data>(Data(0)), X::decrypt(static_cast<uint8_t*>(nullptr), UINT32_C(0), key));
    string name = X::NAME;
  }

private:
  // Type deduction will fail unless the arguments have the same type.
  template <typename T> void same_type(T const&, T const&);
};

}

#endif
