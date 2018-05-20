#pragma once
#ifndef _CPPUTILS_CRYPTO_CRYPTOPP_BYTE_H
#define _CPPUTILS_CRYPTO_CRYPTOPP_BYTE_H

#include <vendor_cryptopp/cryptlib.h>

// If we're running an older CryptoPP version, CryptoPP::byte isn't defined yet.
// Define it. Refer to "byte" type in the global namespace (placed by CryptoPP).
// Could also use CRYPTOPP_NO_GLOBAL_BYTE - but don't want to track when it was
// introduced. This way seems more reliable, as it is compatible with more of
// the Crypto++ versions.
#if CRYPTOPP_VERSION < 600
namespace CryptoPP {
   using byte = ::byte;
}
#endif /* CRYPTOPP_VERSION < 600 */

#endif /* _CPPUTILS_CRYPTO_CRYPTOPP_BYTE_H */
