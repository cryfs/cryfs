#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_INTERVALSET_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_INTERVALSET_H_

#include <cpp-utils/macros.h>
#include <vector>
#include <utility>
#include <functional>
#include <cpp-utils/assert/assert.h>

namespace blockstore {
    namespace caching {

        /**
         * Stores a set of intervals and allows querying whether regions are fully covered by intervals.
         */
        template<class Entry>
        class IntervalSet final {
        public:
            //TODO More efficient implementation (i.e. merging intervals. Not keeping vector<pair>, but sorted vector<Entry> with alternating begin/end entries in the vector).
            IntervalSet();
            IntervalSet(IntervalSet &&rhs) = default;
            IntervalSet &operator=(IntervalSet &&rhs) = default;

            /**
             * Add a new interval
             */
            void add(Entry begin, Entry end);

            /**
             * Returns true, iff the given area is fully covered by intervals
             */
            bool isCovered(Entry begin, Entry end);

            void forEachInterval(std::function<void (Entry begin, Entry end)> callback) const;

        private:
            std::vector<std::pair<Entry, Entry>> _intervals;

            DISALLOW_COPY_AND_ASSIGN(IntervalSet);
        };

        template<class Entry>
        IntervalSet<Entry>::IntervalSet() : _intervals() {
        }

        template<class Entry>
        void IntervalSet<Entry>::add(Entry begin, Entry end) {
            ASSERT(begin <= end, "Invalid interval given");
            _intervals.push_back(std::make_pair(begin, end));
        }

        template<class Entry>
        bool IntervalSet<Entry>::isCovered(Entry begin, Entry end) {
            ASSERT(begin <= end, "Invalid interval given");
            if (begin == end) {
                return true;
            }
            for (const auto &interval : _intervals) {
                if (!(begin < interval.first) && begin < interval.second) {
                    begin = interval.second;
                    if (end <= begin) {
                        return true;
                    }
                } else if (interval.first < end && !(interval.second < end)) {
                    end = interval.first;
                    if (end <= begin) {
                        return true;
                    }
                }
            }
            ASSERT(begin < end, "If begin >= end, we should have stopped earlier.");
            return false;
        }

        template<class Entry>
        void IntervalSet<Entry>::forEachInterval(std::function<void (Entry begin, Entry end)> callback) const {
            for (const auto &interval : _intervals) {
                callback(interval.first, interval.second);
            }
        }
    }
}

#endif
