#include "CryConfigCreator.h"
#include "CryCipher.h"
#include <gitversion/gitversion.h>
#include <cpp-utils/random/Random.h>
#include <cryfs/localstate/LocalStateDir.h>
#include <cryfs/localstate/LocalStateMetadata.h>

using cpputils::Console;
using cpputils::RandomGenerator;
using cpputils::Random;
using std::string;
using std::shared_ptr;
using boost::optional;
using boost::none;

namespace cryfs {

    CryConfigCreator::CryConfigCreator(shared_ptr<Console> console, RandomGenerator &encryptionKeyGenerator, LocalStateDir localStateDir)
        :_console(console), _configConsole(console), _encryptionKeyGenerator(encryptionKeyGenerator), _localStateDir(std::move(localStateDir)) {
    }

    CryConfigCreator::ConfigCreateResult CryConfigCreator::create(const optional<string> &cipherFromCommandLine, const optional<uint32_t> &blocksizeBytesFromCommandLine, const optional<bool> &missingBlockIsIntegrityViolationFromCommandLine, bool allowReplacedFilesystem) {
        CryConfig config;
        config.SetCipher(_generateCipher(cipherFromCommandLine));
        config.SetVersion(CryConfig::FilesystemFormatVersion);
        config.SetCreatedWithVersion(gitversion::VersionString());
        config.SetLastOpenedWithVersion(gitversion::VersionString());
        config.SetBlocksizeBytes(_generateBlocksizeBytes(blocksizeBytesFromCommandLine));
        config.SetRootBlob(_generateRootBlobId());
        config.SetFilesystemId(_generateFilesystemID());
        auto encryptionKey = _generateEncKey(config.Cipher());
        auto localState = LocalStateMetadata::loadOrGenerate(_localStateDir.forFilesystemId(config.FilesystemId()), cpputils::Data::FromString(encryptionKey), allowReplacedFilesystem);
        uint32_t myClientId = localState.myClientId();
        config.SetEncryptionKey(std::move(encryptionKey));
        config.SetExclusiveClientId(_generateExclusiveClientId(missingBlockIsIntegrityViolationFromCommandLine, myClientId));
#ifndef CRYFS_NO_COMPATIBILITY
        config.SetHasVersionNumbers(true);
#endif
        return ConfigCreateResult {std::move(config), myClientId};
    }

    uint32_t CryConfigCreator::_generateBlocksizeBytes(const optional<uint32_t> &blocksizeBytesFromCommandLine) {
        if (blocksizeBytesFromCommandLine != none) {
            // TODO Check block size is valid (i.e. large enough)
            return *blocksizeBytesFromCommandLine;
        } else {
            return _configConsole.askBlocksizeBytes();
        }
    }

    string CryConfigCreator::_generateCipher(const optional<string> &cipherFromCommandLine) {
        if (cipherFromCommandLine != none) {
            ASSERT(std::find(CryCiphers::supportedCipherNames().begin(), CryCiphers::supportedCipherNames().end(), *cipherFromCommandLine) != CryCiphers::supportedCipherNames().end(), "Invalid cipher");
            return *cipherFromCommandLine;
        } else {
            return _configConsole.askCipher();
        }
    }

    optional<uint32_t> CryConfigCreator::_generateExclusiveClientId(const optional<bool> &missingBlockIsIntegrityViolationFromCommandLine, uint32_t myClientId) {
        if (!_generateMissingBlockIsIntegrityViolation(missingBlockIsIntegrityViolationFromCommandLine)) {
            return none;
        }
        return myClientId;
    }

    bool CryConfigCreator::_generateMissingBlockIsIntegrityViolation(const optional<bool> &missingBlockIsIntegrityViolationFromCommandLine) {
        if (missingBlockIsIntegrityViolationFromCommandLine != none) {
            return *missingBlockIsIntegrityViolationFromCommandLine;
        } else {
            return _configConsole.askMissingBlockIsIntegrityViolation();
        }
    }

    string CryConfigCreator::_generateEncKey(const std::string &cipher) {
        _console->print("\nGenerating secure encryption key. This can take some time...");
        auto key = CryCiphers::find(cipher).createKey(_encryptionKeyGenerator);
        _console->print("done\n");
        return key;
    }

    string CryConfigCreator::_generateRootBlobId() {
        //An empty root blob entry will tell CryDevice to create a new root blob
        return "";
    }

    CryConfig::FilesystemID CryConfigCreator::_generateFilesystemID() {
        return Random::PseudoRandom().getFixedSize<CryConfig::FilesystemID::BINARY_LENGTH>();
    }
}
