#include "utils.h"
#include <algorithm>

using std::pair;
using std::make_pair;
using std::vector;
using std::string;

namespace cryfs {
    namespace program_options {
        pair<vector<char*>, vector<char*>> splitAtDoubleDash(const vector<char*> &options) {
            auto doubleDashIterator = std::find(options.begin(), options.end(), string("--"));
            vector<char*> beforeDoubleDash(options.begin(), doubleDashIterator);
            vector<char*> afterDoubleDash;
            afterDoubleDash.reserve(options.size()-beforeDoubleDash.size()+1);
            afterDoubleDash.push_back(options[0]);
            std::copy(doubleDashIterator+1, options.end(), std::back_inserter(afterDoubleDash));
            return make_pair(
                    beforeDoubleDash,
                    afterDoubleDash
            );
        }

    }
}
