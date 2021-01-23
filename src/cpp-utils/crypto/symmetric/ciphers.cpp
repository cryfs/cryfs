#include "ciphers.h"

#define DEFINE_CIPHER(InstanceName)                                   \
    constexpr const char *InstanceName::NAME;                         \

namespace cpputils {

    DEFINE_CIPHER(XChaCha20Poly1305);

    DEFINE_CIPHER(AES256_GCM);
    DEFINE_CIPHER(AES256_CFB);
    DEFINE_CIPHER(AES128_GCM);
    DEFINE_CIPHER(AES128_CFB);

    DEFINE_CIPHER(Twofish256_GCM);
    DEFINE_CIPHER(Twofish256_CFB);
    DEFINE_CIPHER(Twofish128_GCM);
    DEFINE_CIPHER(Twofish128_CFB);

    DEFINE_CIPHER(Serpent256_GCM);
    DEFINE_CIPHER(Serpent256_CFB);
    DEFINE_CIPHER(Serpent128_GCM);
    DEFINE_CIPHER(Serpent128_CFB);

    DEFINE_CIPHER(Cast256_GCM);
    DEFINE_CIPHER(Cast256_CFB);

    DEFINE_CIPHER(Mars448_GCM);
    DEFINE_CIPHER(Mars448_CFB);
    DEFINE_CIPHER(Mars256_GCM);
    DEFINE_CIPHER(Mars256_CFB);
    DEFINE_CIPHER(Mars128_GCM);
    DEFINE_CIPHER(Mars128_CFB);

}
