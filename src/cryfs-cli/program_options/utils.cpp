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
            const vector<string> beforeDoubleDash(options.begin(), doubleDashIterator);
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

        namespace {
            constexpr const char *NONEMPTY_OPTION = "nonempty";

            // Removes 'nonempty' from a comma separated list of mount options and returns whether it was there.
            bool extractNonemptyOptionFromCsv(string *csvOptions) {
                bool found = false;
                string remaining;
                size_t pos = 0;
                while (pos <= csvOptions->size()) {
                    const size_t next = csvOptions->find(',', pos);
                    const size_t end = (next == string::npos) ? csvOptions->size() : next;
                    const string option = csvOptions->substr(pos, end - pos);
                    if (option == NONEMPTY_OPTION) {
                        found = true;
                    } else {
                        if (!remaining.empty()) {
                            remaining += ",";
                        }
                        remaining += option;
                    }
                    if (next == string::npos) {
                        break;
                    }
                    pos = next + 1;
                }
                *csvOptions = std::move(remaining);
                return found;
            }
        }

        bool extractNonemptyOption(vector<string> *fuseOptions) {
            bool found = false;
            bool lastOptionWasDashO = false;
            size_t i = 0;
            while (i < fuseOptions->size()) {
                if (lastOptionWasDashO) {
                    // A '-o' is always followed by its value, so i >= 1 here.
                    if (extractNonemptyOptionFromCsv(&(*fuseOptions)[i])) {
                        found = true;
                    }
                    if ((*fuseOptions)[i].empty()) {
                        // 'nonempty' was the only option in this argument. Remove the now empty
                        // argument together with the '-o' before it, because libfuse rejects a '-o'
                        // that has no value.
                        fuseOptions->erase(fuseOptions->begin() + i - 1, fuseOptions->begin() + i + 1);
                        i -= 1;
                        lastOptionWasDashO = false;
                        continue;
                    }
                }
                lastOptionWasDashO = ((*fuseOptions)[i] == "-o");
                ++i;
            }
            return found;
        }
    }
}
