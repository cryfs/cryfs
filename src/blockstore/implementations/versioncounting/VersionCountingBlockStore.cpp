#include "VersionCountingBlockStore.h"
#include "VersionCountingBlock.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using boost::none;
using boost::optional;
namespace bf = boost::filesystem;

namespace blockstore {
    namespace versioncounting {

        VersionCountingBlockStore::VersionCountingBlockStore(unique_ref<BlockStore> baseBlockStore, const bf::path &integrityFilePath, bool missingBlockIsIntegrityViolation)
                : _baseBlockStore(std::move(baseBlockStore)), _knownBlockVersions(integrityFilePath), _missingBlockIsIntegrityViolation(missingBlockIsIntegrityViolation), _integrityViolationDetected(false) {
        }

        Key VersionCountingBlockStore::createKey() {
            return _baseBlockStore->createKey();
        }

        optional<unique_ref<Block>> VersionCountingBlockStore::tryCreate(const Key &key, cpputils::Data data) {
            _checkNoPastIntegrityViolations();
            //TODO Easier implementation? This is only so complicated because of the cast VersionCountingBlock -> Block
            auto result = VersionCountingBlock::TryCreateNew(_baseBlockStore.get(), key, std::move(data), this);
            if (result == boost::none) {
                return boost::none;
            }
            return unique_ref<Block>(std::move(*result));
        }

        optional<unique_ref<Block>> VersionCountingBlockStore::load(const Key &key) {
            _checkNoPastIntegrityViolations();
            auto block = _baseBlockStore->load(key);
            if (block == boost::none) {
                if (_missingBlockIsIntegrityViolation && _knownBlockVersions.blockShouldExist(key)) {
                    integrityViolationDetected();
                }
                return boost::none;
            }
            return optional<unique_ref<Block>>(VersionCountingBlock::Load(std::move(*block), this));
        }

        void VersionCountingBlockStore::_checkNoPastIntegrityViolations() {
            if (_integrityViolationDetected) {
                throw std::runtime_error(string() +
                                         "There was an integrity violation detected. Preventing any further access to the file system. " +
                                         "If you want to reset the integrity data (i.e. accept changes made by a potential attacker), " +
                                         "please unmount the file system and delete the following file before re-mounting it: " +
                                         _knownBlockVersions.path().native());
            }
        }

        void VersionCountingBlockStore::integrityViolationDetected() {
            _integrityViolationDetected = true;
            throw IntegrityViolationError("A block that should exist wasn't found. Did an attacker delete it?");
        }

        void VersionCountingBlockStore::remove(unique_ref<Block> block) {
            Key key = block->key();
            auto versionCountingBlock = cpputils::dynamic_pointer_move<VersionCountingBlock>(block);
            ASSERT(versionCountingBlock != boost::none, "Block is not an VersionCountingBlock");
            _knownBlockVersions.markBlockAsDeleted(key);
            auto baseBlock = (*versionCountingBlock)->releaseBlock();
            _baseBlockStore->remove(std::move(baseBlock));
        }

        uint64_t VersionCountingBlockStore::numBlocks() const {
            return _baseBlockStore->numBlocks();
        }

        uint64_t VersionCountingBlockStore::estimateNumFreeBytes() const {
            return _baseBlockStore->estimateNumFreeBytes();
        }

        uint64_t VersionCountingBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
            return VersionCountingBlock::blockSizeFromPhysicalBlockSize(_baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize));
        }

        void VersionCountingBlockStore::forEachBlock(std::function<void (const Key &)> callback) const {
            return _baseBlockStore->forEachBlock(callback);
        }

#ifndef CRYFS_NO_COMPATIBILITY
        void VersionCountingBlockStore::migrateFromBlockstoreWithoutVersionNumbers(BlockStore *baseBlockStore, const bf::path &integrityFilePath) {
            std::cout << "Migrating file system for integrity features. This can take a while..." << std::flush;
            KnownBlockVersions knownBlockVersions(integrityFilePath);
            baseBlockStore->forEachBlock([&baseBlockStore, &knownBlockVersions] (const Key &key) {
                auto block =  baseBlockStore->load(key);
                ASSERT(block != none, "Couldn't load block for migration");
                VersionCountingBlock::migrateFromBlockstoreWithoutVersionNumbers(std::move(*block), &knownBlockVersions);
            });
            std::cout << "done" << std::endl;
        }
#endif

    }
}
