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
template<class Context, class... Args>
class context_list final {
public:
  context_list();

  Context *create(Args... args);
  cryfs_status remove(Context *ctx);

private:
  std::mutex _mutex;
  std::vector<cpputils::unique_ref<Context>> _contexts;

  DISALLOW_COPY_AND_ASSIGN(context_list);
};

template<class Context, class... Args>
inline context_list<Context, Args...>::context_list()
    : _mutex(), _contexts() {
};

template<class Context, class... Args>
Context *context_list<Context, Args...>::create(Args... args) {
  std::unique_lock<std::mutex> lock(_mutex);

  auto context_ref = cpputils::make_unique_ref<Context>(args...);
  Context* result = context_ref.get();
  _contexts.push_back(std::move(context_ref));
  return result;
}

template<class Context, class... Args>
cryfs_status context_list<Context, Args...>::remove(Context* ctx) {
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
