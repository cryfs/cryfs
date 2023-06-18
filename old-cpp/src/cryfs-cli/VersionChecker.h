#ifndef MESSMER_CRYFSCLI_VERSIONCHECKER_H
#define MESSMER_CRYFSCLI_VERSIONCHECKER_H

#include <cpp-utils/macros.h>
#include <string>
#include <boost/optional.hpp>
#include <boost/property_tree/ptree.hpp>
#include <cpp-utils/network/HttpClient.h>
#include <cpp-utils/pointer/unique_ref.h>

namespace cryfs_cli {
    class VersionChecker final {
    public:
        //TODO Write a cpputils::shared_ref and use it
        VersionChecker(cpputils::HttpClient* httpClient);

        boost::optional<std::string> newestVersion() const;
        boost::optional<std::string> securityWarningFor(const std::string &version) const;
    private:
        static boost::optional<boost::property_tree::ptree> _getVersionInfo(cpputils::HttpClient* httpClient);
        static boost::optional<boost::property_tree::ptree> _parseJson(const std::string &json);

        boost::optional<boost::property_tree::ptree> _versionInfo;

        DISALLOW_COPY_AND_ASSIGN(VersionChecker);
    };
}

#endif
