#pragma once
#ifndef MESSMER_BLOCKSTORE_UTILS_IDWRAPPER_H_
#define MESSMER_BLOCKSTORE_UTILS_IDWRAPPER_H_

#include <string>
#include <cpp-utils/data/FixedSizeData.h>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/data/SerializationHelper.h>

namespace blockstore {

// Tag is used to distinguish different concrete IdWrappers
template<class Tag>
class IdWrapper final {
private:
  using IdData = cpputils::FixedSizeData<16>;

public:
  static constexpr size_t BINARY_LENGTH = IdData::BINARY_LENGTH;
  static constexpr size_t STRING_LENGTH = IdData::STRING_LENGTH;

  explicit IdWrapper(const IdData& id);
  const IdData& data() const;

  static IdWrapper Random();
  static IdWrapper Null();

  static IdWrapper FromString(const std::string &data);
  std::string ToString() const;

  static IdWrapper FromBinary(const void *source);
  void ToBinary(void *target) const;

private:

  IdData id_;
  friend struct std::hash<IdWrapper>;
  friend struct std::less<IdWrapper>;
  template<class Tag2> friend bool operator==(const IdWrapper<Tag2>& lhs, const IdWrapper<Tag2>& rhs);
  template<class Tag2> friend bool operator!=(const IdWrapper<Tag2>& lhs, const IdWrapper<Tag2>& rhs);
};

template<class Tag>
constexpr size_t IdWrapper<Tag>::BINARY_LENGTH;

template<class Tag>
constexpr size_t IdWrapper<Tag>::STRING_LENGTH;

template<class Tag>
inline IdWrapper<Tag>::IdWrapper(const IdData& id): id_(id) {}

template<class Tag>
inline IdWrapper<Tag> IdWrapper<Tag>::Random() {
    return IdWrapper(cpputils::Random::PseudoRandom().getFixedSize<BINARY_LENGTH>());
}

template<class Tag>
inline IdWrapper<Tag> IdWrapper<Tag>::Null() {
    return IdWrapper(IdData::Null());
}

template<class Tag>
inline IdWrapper<Tag> IdWrapper<Tag>::FromString(const std::string &data) {
    return IdWrapper(IdData::FromString(data));
}

template<class Tag>
inline std::string IdWrapper<Tag>::ToString() const {
    return id_.ToString();
}

template<class Tag>
inline IdWrapper<Tag> IdWrapper<Tag>::FromBinary(const void *source) {
    return IdWrapper(IdData::FromBinary(source));
}

template<class Tag>
inline void IdWrapper<Tag>::ToBinary(void *target) const {
    id_.ToBinary(target);
}

template<class Tag>
inline const typename IdWrapper<Tag>::IdData& IdWrapper<Tag>::data() const {
    return id_;
}

template<class Tag>
inline bool operator==(const IdWrapper<Tag>& lhs, const IdWrapper<Tag>& rhs) {
  return lhs.id_ == rhs.id_;
}

template<class Tag>
inline bool operator!=(const IdWrapper<Tag>& lhs, const IdWrapper<Tag>& rhs) {
  return !operator==(lhs, rhs);
}

}

#define DEFINE_IDWRAPPER(IdWrapper)                                                                                    \
  namespace std {                                                                                                      \
    /*Allow using IdWrapper in std::unordered_map / std::unordered_set */                                              \
    template <> struct hash<IdWrapper> {                                                                               \
      size_t operator()(const IdWrapper &idWrapper) const {                                                            \
        /*Ids are random, so it is enough to use the first few bytes as a hash */                                      \
        return cpputils::deserialize<size_t>(idWrapper.id_.data());                                                    \
      }                                                                                                                \
    };                                                                                                                 \
    /*Allow using IdWrapper in std::map / std::set */                                                                  \
    template <> struct less<IdWrapper> {                                                                               \
      bool operator()(const IdWrapper &lhs, const IdWrapper &rhs) const {                                              \
        return 0 > std::memcmp(lhs.id_.data(), rhs.id_.data(), IdWrapper::BINARY_LENGTH);                              \
      }                                                                                                                \
    };                                                                                                                 \
  }                                                                                                                    \

#endif
