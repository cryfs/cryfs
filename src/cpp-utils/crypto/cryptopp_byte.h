#pragma once
#ifndef _CRYPTOPP_BYTE_H
#define _CRYPTOPP_BYTE_H

#include <cryptopp/cryptlib.h>

// If we're running an older CryptoPP version, CryptoPP::byte isn't defined yet. Define it.
#if CRYPTOPP_VERSION < 600
namespace CryptoPP {
   using byte = ::byte;
}
#endif

#endif /* _CRYPTOPP_BYTE_H */
