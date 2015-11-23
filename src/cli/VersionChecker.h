#ifndef MESSMER_CRYFS_SRC_CLI_VERSIONCHECKER_H
#define MESSMER_CRYFS_SRC_CLI_VERSIONCHECKER_H

#include <messmer/cpp-utils/macros.h>
#include <string>
#include <boost/optional.hpp>
#include <boost/property_tree/ptree.hpp>

namespace cryfs {
    //TODO Test

    class VersionChecker final {
    public:
        VersionChecker();

        boost::optional<std::string> newestVersion() const;
        boost::optional<std::string> securityWarningFor(const std::string &version) const;
    private:
        static boost::optional<boost::property_tree::ptree> _getVersionInfo();
        static boost::optional<boost::property_tree::ptree> _parseJson(const std::string &json);

        boost::optional<boost::property_tree::ptree> _versionInfo;

        DISALLOW_COPY_AND_ASSIGN(VersionChecker);
    };
}

#endif
