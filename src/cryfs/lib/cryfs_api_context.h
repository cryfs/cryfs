#pragma once
#ifndef CRYFS_CRYFS_API_CONTEXT_H
#define CRYFS_CRYFS_API_CONTEXT_H

#include <vector>
#include <cpp-utils/pointer/unique_ref.h>
#include <mutex>
#include "context_list.h"
#include "../cryfs.h"

struct cryfs_api_context final {
public:
  cryfs_api_context();

  cryfs_load_context *new_load_context();
  cryfs_create_context *new_create_context();

  cryfs_status delete_load_context(cryfs_load_context *context);
  cryfs_status delete_create_context(cryfs_create_context *context);

private:
  cryfs::context_list<cryfs_load_context> _load_contexts;
  cryfs::context_list<cryfs_create_context> _create_contexts;

  DISALLOW_COPY_AND_ASSIGN(cryfs_api_context);
};

#endif
