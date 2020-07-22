#include "VersionChecker.h"
#include <sstream>
#include <cpp-utils/network/CurlHttpClient.h>
#include <boost/property_tree/json_parser.hpp>
#include <cpp-utils/logging/logging.h>
#include <boost/foreach.hpp>

using boost::optional;
using boost::none;
using std::string;
using cpputils::HttpClient;
using boost::property_tree::ptree;
using boost::property_tree::json_parser_error;
using namespace cpputils::logging;

namespace cryfs_cli {

    VersionChecker::VersionChecker(HttpClient* httpClient)
            : _versionInfo(_getVersionInfo(httpClient)) {}

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
        // NOLINTNEXTLINE(bugprone-branch-clone)
        BOOST_FOREACH(const ptree::value_type &v, *warnings) {
            if(v.first == version) {
                return v.second.get_value<std::string>();
            }
        }
        return none;
    }

    optional<ptree> VersionChecker::_getVersionInfo(HttpClient* httpClient) {
        long timeoutMsec = 2000;
		string response;
		try {
			response = httpClient->get("https://www.cryfs.org/version_info.json", timeoutMsec);
		}
		catch (const std::exception& e) {
			LOG(WARN, "HTTP Error: {}", e.what());
			return none;
		}
        return _parseJson(response);
    }

    optional<ptree> VersionChecker::_parseJson(const string &json) {
        try {
            ptree pt;
            std::istringstream input(json);
            read_json(input, pt);
            return pt;
        } catch (const json_parser_error &e) {
            LOG(WARN, "Error parsing version information json object");
            return none;
        }
    }

}
