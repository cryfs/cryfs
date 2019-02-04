#pragma once
#ifndef CRYFS_CONTEXT_LIST_H
#define CRYFS_CONTEXT_LIST_H

#include <cpp-utils/pointer/unique_ref.h>
#include <vector>
#include <functional>
#include <mutex>
#include "../cryfs.h"

namespace cryfs {

// This class keeps ownership of contexts and destroys them in its destructor.
template<class Context>
class context_list final {
public:
  context_list();

  template<class... Args> Context *create(Args&&... args);
  cryfs_status remove(Context *ctx);

private:
  std::mutex _mutex;
  std::vector<cpputils::unique_ref<Context>> _contexts;

  DISALLOW_COPY_AND_ASSIGN(context_list);
};

template<class Context>
inline context_list<Context>::context_list()
    : _mutex(), _contexts() {
};

template<class Context>
template<class... Args>
Context *context_list<Context>::create(Args&&... args) {
  static_assert(std::is_constructible<Context, Args...>::value, "Wrong arguments for Context constructor");

  std::unique_lock<std::mutex> lock(_mutex);

  auto context_ref = cpputils::make_unique_ref<Context>(std::forward<Args>(args)...);
  Context* result = context_ref.get();
  _contexts.push_back(std::move(context_ref));
  return result;
}

template<class Context>
cryfs_status context_list<Context>::remove(Context* ctx) {
  std::unique_lock<std::mutex> lock(_mutex);

  auto found = std::find_if(_contexts.begin(), _contexts.end(), [&] (const auto& element) {return element.get() == ctx;});
  if (found == _contexts.end()) {
    return cryfs_error_INVALID_CONTEXT;
  }
  _contexts.erase(found);
  return cryfs_success;
}

}

#endif
