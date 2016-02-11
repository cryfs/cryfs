#include "VersionChecker.h"
#include <sstream>
#include <cpp-utils/network/CurlHttpClient.h>
#include <boost/property_tree/json_parser.hpp>
#include <cpp-utils/logging/logging.h>
#include <boost/foreach.hpp>

using boost::optional;
using boost::none;
using std::string;
using std::shared_ptr;
using std::make_shared;
using cpputils::HttpClient;
using cpputils::CurlHttpClient;
using boost::property_tree::ptree;
using boost::property_tree::json_parser_error;
using namespace cpputils::logging;

namespace cryfs {

    VersionChecker::VersionChecker(shared_ptr<HttpClient> httpClient)
            : _versionInfo(_getVersionInfo(std::move(httpClient))) {
    }

    optional<string> VersionChecker::newestVersion() const {
        if (_versionInfo == none) {
            return none;
        }
        string version = _versionInfo->get("version_info.current", "");
        if (version == "") {
            return none;
        }
        return version;
    }

    optional<string> VersionChecker::securityWarningFor(const string &version) const {
        if (_versionInfo == none) {
            return none;
        }
        auto warnings = _versionInfo->get_child_optional("warnings");
        if (warnings == none) {
            return none;
        }
        BOOST_FOREACH(const ptree::value_type &v, *warnings) {
            if(v.first == version) {
                return v.second.get_value<std::string>();
            }
        }
        return none;
    }

    optional<ptree> VersionChecker::_getVersionInfo(shared_ptr<HttpClient> httpClient) {
        long timeoutMsec = 2000;
        optional<string> response = httpClient->get("https://www.cryfs.org/version_info.json", timeoutMsec);
        if (response == none) {
            return none;
        }
        return _parseJson(*response);
    }

    optional<ptree> VersionChecker::_parseJson(const string &json) {
        try {
            ptree pt;
            std::istringstream input(json);
            read_json(input, pt);
            return pt;
        } catch (const json_parser_error &e) {
            LOG(WARN) << "Error parsing version information json object";
            return none;
        }
    }

}
