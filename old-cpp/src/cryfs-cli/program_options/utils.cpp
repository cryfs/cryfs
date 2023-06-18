#include "utils.h"
#include <algorithm>
#include <string>
#include <iterator>

using std::pair;
using std::make_pair;
using std::vector;
using std::string;

namespace cryfs_cli {
    namespace program_options {
        pair<vector<string>, vector<string>> splitAtDoubleDash(const vector<string> &options) {
            auto doubleDashIterator = std::find(options.begin(), options.end(), string("--"));
            vector<string> beforeDoubleDash(options.begin(), doubleDashIterator);
            vector<string> afterDoubleDash;
            if (doubleDashIterator != options.end() && doubleDashIterator + 1 != options.end()) {
                afterDoubleDash.reserve(options.size() - beforeDoubleDash.size() - 1);
                std::copy(doubleDashIterator + 1, options.end(), std::back_inserter(afterDoubleDash));
            }
            return make_pair(
                    beforeDoubleDash,
                    afterDoubleDash
            );
        }

    }
}
