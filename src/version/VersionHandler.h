#ifndef MESSMER_CRYFS_VERSIONHANDLER_H
#define MESSMER_CRYFS_VERSIONHANDLER_H

#include "Version.h"
#include "VersionParser.h"

namespace git_version_builder {
#include "git_version.h"
}

namespace version {
    constexpr unsigned int COMMITS_SINCE_TAG = git_version_builder::version::COMMITS_SINCE_TAG;
    constexpr const char *GIT_COMMIT_ID = git_version_builder::version::GIT_COMMIT_ID;
    constexpr const Version VERSION = VersionParser::parse(git_version_builder::version::TAG_NAME,
                                                           git_version_builder::version::COMMITS_SINCE_TAG,
                                                           git_version_builder::version::GIT_COMMIT_ID);
}

#endif
