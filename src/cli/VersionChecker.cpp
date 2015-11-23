#include "VersionChecker.h"
#include <sstream>
#include "HttpClient.h"
#include <boost/property_tree/json_parser.hpp>
#include <messmer/cpp-utils/logging/logging.h>
#include <boost/foreach.hpp>

using boost::optional;
using boost::none;
using std::string;
using boost::property_tree::ptree;
using boost::property_tree::json_parser_error;
using namespace cpputils::logging;

#include <iostream>
namespace cryfs {

    VersionChecker::VersionChecker(): _versionInfo(_getVersionInfo()) {
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
        BOOST_FOREACH(const ptree::value_type &v, _versionInfo->get_child("warnings")) {
            if(v.first == version) {
                return v.second.get_value<std::string>();
            }
        }
        return none;
    }

    optional<ptree> VersionChecker::_getVersionInfo() {
        optional<string> response = HttpClient().get("http://www.cryfs.org/version_info.json");
        if (response == none) {
            std::cout << "no response" << std::endl;
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
