#include <gtest/gtest.h>
#include <vector>
#include <boost/filesystem.hpp>
#include <cpp-utils/data/Data.h>
#include <cryptopp-vendor/hex.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cryfs/impl/config/CryConfigFile.h>
#include <cryfs/impl/config/CryPresetPasswordBasedKeyProvider.h>
#include "../../impl/testutils/MockConsole.h"

using cpputils::Data;
using cpputils::AES256_GCM;
using cpputils::Serpent128_CFB;
using cpputils::TempFile;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::SCrypt;

using namespace cryfs;

// Test that config files created with (old) versions of cryfs are still loadable.
// The config files created with these cryfs versions are included in the test cases in hex code.
// To make tests run fast, the config files should be created using SCrypt::TestSettings.
class CryConfigCompatibilityTest: public ::testing::Test {
public:
    TempFile file;

    unique_ref<CryConfigFile> loadConfigFromHex(const string &configFileContentHex) {
        storeHexToFile(configFileContentHex);
        CryPresetPasswordBasedKeyProvider keyProvider("mypassword", make_unique_ref<SCrypt>(SCrypt::DefaultSettings));
        return CryConfigFile::load(file.path(), &keyProvider, CryConfigFile::Access::ReadWrite).right();
    }

private:

    void storeHexToFile(const string &hex) {
        hexToBinary(hex).StoreToFile(file.path());
    }

    Data hexToBinary(const string &hex) {
        ASSERT(hex.size()%2 == 0, "Hex codes need to have two characters per byte");
        Data result(hex.size()/2);
        {
            const CryptoPP::StringSource _1(hex, true,
                 new CryptoPP::HexDecoder(
                       new CryptoPP::ArraySink(static_cast<CryptoPP::byte*>(result.data()), result.size())
                 )
            );
        }
        return result;
    }

};


#ifndef CRYFS_NO_COMPATIBILITY

TEST_F(CryConfigCompatibilityTest, v0_8_1_with_aes_256_gcm) {
    auto config = loadConfigFromHex("63727966732e636f6e6669673b303b736372797074000004000000000000010000000100000020000000000000000af023e55f804ad303c1dbdf6a2b2bed8cc1ab3d0b2f3312c073628dc041e6f3b92f54fad63a1d4e0ad47a33e65e08080dd4e5bdec0a95fe777705ad68e88eabcb4b91d4f25c32e3e44d6e893421c9efa6d4b2c56d66d0546a410de489b04160b276184ebe7dd77840ce6e414f829bb4399451e055d3406261b4faf5fb52f27e21823f8073df255f96b37edf2c2e58ecee21ae82e3883341024aff326cc4c0929f0ed90473511cbe801d1e34899b3cdb9556477be0127c35375967bf7c8392ad4d30d81479c7923900f532b195b71cff868fa1643d1f9e0a0d7996260691d7b5b8ef3c319c2f2303f822eac389e1d6822e7ee57df60c922fa10fbc7e5970e723df2fdb2a6529c94ea1d2507f55cb9bc3fecab77bf3102e96c6675a618b54bb18e632dd99d3d47fa9e24392535abc8c76f891df76c2193233d6c93851966f69def3f53108be83b30980c98e377186da71297a1b61df76e9f138a3998577a0c486452f1b6a2d89b1e62d78695ac6a062d5a4cf5972de7bc2775fd27d25337f0666250f19d6fe50fe762371befa26de6a9299d7df362780f59ccc475e28c71cc5af9e817e4e2160005393ec7fb385d467e0915e7169934ea986141101c9e6b9c62b8819c51a4422e7fe58e24c3293c3f12173a4c480334cb67c9313df667c65802fb778e20f5a4d6cfd07fffb35c2ae8fdf235882bf0e6371125509872e73bd1bc3faca72948db76d8179e08bf3391bafdede591d5623240ba072ada17c2444116bf8c08ee32367eb86eda02ecfecc625b4bc974428b8f5024df430c352b9849f48774bac1455e7364f7660abc039d2a1134cde596a1e7c3a1a8d68591e8640a2d308e2ea5f03874ba1fa8e9ffec2d1bc1541d124adcd3e9f4ed23343e2148629317e6304c63c2a5ceeeb572246393c8e50196728e0b604aa9e2d49e65680cc53ee343645412239f82500ef3d70a6b6e3335ef8cbcfe4fd46b84f634f6fe96cd1b79207a2120ac2bad668bcdbc377b98448c9afbbda9fe8164dbb48aaa68b0aaa90e52bc419ea2ea1f5e12bb649831af12f1ef14765f312640e22c5508531bd46e3ab3a7f922a91fb83332104609e385d61d3778bc5acb03bee522ea5b791d27026fd589ec2b32f04caaf2ef2effbd50e12430ef42631fdabd061966fde1489b9a1eff3fe40b295d75e36af19ae0e69559b9c81b3bc47b35fef5c7038b83db6729aac522efee897a44895804ffbeb11317a78586f8a3511905514eee86d7cf0a395f5f0af521c72542e544ff359a69150a5e64f00b020d48c20d5716331c3b0ac314eeceaadf8b73516b4273b48046b5c3a3980bade0cc25baf54afb27755d155d4b94fa556deb30d58b1d920cbe01b13479135ff84d6a9345a0d418075ecc66bd10d8d2a07963450256897018f54b2a88fffe9383913565823dbfb71568f5d9af93e542937ecde25c2d4407ad964ece5c09a1f81a57c18a8bb2aa8d9d062415903c98752d70ad7175d0d30b391da77459e9a68a69825b3bb4061bc2504789fc395");
    EXPECT_EQ(AES256_GCM::NAME, config->config()->Cipher());
    EXPECT_EQ("4A70AD8AC565A5123586D0928B496465", config->config()->RootBlob());
}

TEST_F(CryConfigCompatibilityTest, v0_8_1_with_serpent_128_cfb) {
    auto config = loadConfigFromHex("63727966732e636f6e6669673b303b736372797074000004000000000000010000000100000020000000000000006496611b94011d03433181e83e80c19888a5607392c2117cd377251978fbbf8d46716bd90dbae3c1728b40e7dc086ee97b8a641f1ea6cc31ccda9a1e4a7a1bf4b6ca3d1d0075acba94017c034e87dfad048c9448df24ea7b89b14d60c9f9e0162cfeb420a85759aaa011232cff99e81b25c456cb13690d5cc735e61ef9b01800961a90488073c5320c14a562f0eccd32383d144476d72d9c61733fb3561f5187c2d6069ce7ff4c43df4fc3f7c1d63df239d05a2b377f70816fb31f7a65d8fa667cd14b62a67e2d67adbf841860af2abed9d4c839c510c7fac7cd58210238830a4926917d663500dd3dc2395845297b3726c00228cb08a2bde648d2e9bc2a3e8655f86bc2f5c2b327e71c75b92894dd43b56dfefeeea598fbae9782dca64cc9676df3a954a9ecedbe3f8fe16e790aeec2ac2336984c8fcce74bfebf310d6fdcc0882ee7f2bdd32928aa53b09084e8035ec235ebe971a0b141e5fcab6ce2207f0f399e683ff4994ed638194cc7d3891019a2a674d14180114494e21270e4234a51e28661d4ac908d4f2b2d7c94ffcb74df8c9f08fb68d9a7167a05fae370e041689cce5fb13fdf3ff14863f6c882e6d302eafe5348362c026853a3717756cefb4657a721efab95f6d949b508a98de86885fe37b2f16f1792411ff47c1074e2c2d301e6401f251854b28c329040c8e55184402648bc79710536390b0f99acd2400734a81e8bd573d9a96971a5ef195d85babbe1092e271a69a146d2d4114b855c89b710908d4a584e598b6dd186709d956fbd8b13b2ba3e9115c7d75416b7ac0e6d9dde83d367af00fa8dae54f03079cdf5fa1bac35ff3fbf6440f4fa41d06278fd8a6c3edc80d4ceee900975db2a345d8eda0ce0260d389a8336134a20945c1dde16e475ae537a4407da7f389dce79b439d0d81fa915a7dc4cb018e0d1bd806101ff53970731876850255dd5b7d78bf817112bc74ebae39ecae0aa1555e2a9e61660f4ba373086492e0561efb3bfa98399f899d6909415f19963cef3dc6d154056403ed1f475cbc78981cc949c23686bd71b420ed7ce654578d322f1eeec7413c77c017479f2e19b0c5ebb25ad76e1af8ec3a6b6ba3e092af8ddd624dfd1fccb4a61cc77f5c001fafdd7d2cc6c7b418c6dac97e840bc027ef9d4b237eebc2d1255bc0f796c3de78e31a9ae346734801e0f51dc47f8279164ab35bad3b06a5f74356deec0296c350aae4da560b8d77782195621dca054703433037f28cac2603cdefe52903f25013ad4cf4f29e0e413f89f950d26b4ef129aabf78abf9acae252bc60db373cfe3963fc87e9532056f2f2a7f9f1ddd1a3cf9d0078a5c9cccdcc60714de967556032283075424446ccda9f62a5e31f2b5efbc2256c742e4a300d5bc797e90b3a65fdffe3c5f22f5a885903331dbbb031e51411facb4e80e1728c5b14c9fce196d7d5f1dfeca2fe9b877f45fc1f1c8ec09dd5335eb89accf38fe0e6dbbe77a59c96a49830ce5279f17f4897c64e2aeaec127d8d06adcefa7ac800c1e8");
    EXPECT_EQ(Serpent128_CFB::NAME, config->config()->Cipher());
    EXPECT_EQ("5821ED3B0739C7EDB6A09590232EA75B", config->config()->RootBlob());
}

#endif
