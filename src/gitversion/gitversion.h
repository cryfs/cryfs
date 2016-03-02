#pragma once
#ifndef GITVERSION_GITVERSION_H
#define GITVERSION_GITVERSION_H

#include "versionstring.h"

namespace gitversion {
    bool IsDevVersion();
    bool IsStableVersion();
    std::string MajorVersion();
    std::string MinorVersion();
    std::string GitCommitId();
}

#endif
