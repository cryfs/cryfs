#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_MYCLIENTID_H_
#define MESSMER_CRYFS_LOCALSTATE_MYCLIENTID_H_

#include <cpp-utils/macros.h>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>

namespace cryfs {

    class MyClientId final {
    public:
        MyClientId(const boost::filesystem::path &statePath);

        uint32_t loadOrGenerate() const;

    private:
        const boost::filesystem::path _stateFilePath;

        static uint32_t _generate();
        boost::optional<uint32_t> _load() const;
        void _save(uint32_t clientId) const;

        DISALLOW_COPY_AND_ASSIGN(MyClientId);
    };

}


#endif
