#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_INTERVALSET_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_INTERVALSET_H_

#include <cpp-utils/macros.h>
#include <vector>
#include <utility>
#include <functional>
#include <cpp-utils/assert/assert.h>
#include <algorithm>

namespace blockstore {
    namespace caching {

        /**
         * Stores a set of intervals and allows querying whether regions are fully covered by intervals.
         */
        template<class Entry>
        class IntervalSet final {
        public:
            //TODO Test cases for different merges in add()
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

            void _mergeRight(typename std::vector<std::pair<Entry,Entry>>::iterator pos);
            bool _intervalsDontOverlap() const;

            DISALLOW_COPY_AND_ASSIGN(IntervalSet);
        };

        template<class Entry>
        IntervalSet<Entry>::IntervalSet() : _intervals() {
        }

        template<class Entry>
        void IntervalSet<Entry>::add(Entry begin, Entry end) {
            ASSERT(begin <= end, "Invalid interval given");
            if (begin < end) {
                auto insertPos = std::find_if(_intervals.begin(), _intervals.end(), [begin] (const auto &entry) {return begin < entry.first;});
                auto newElem = _intervals.insert(insertPos, std::make_pair(begin, end));
                auto firstPossiblyInvalidEntry = (newElem == _intervals.begin()) ? _intervals.begin() : (newElem-1);
                _mergeRight(firstPossiblyInvalidEntry);
                ASSERT(_intervalsDontOverlap(), "Intervals shouldn't overlap");
                ASSERT(isCovered(begin, end), "Added region should be covered");
            }
        }

        template<class Entry>
        void IntervalSet<Entry>::_mergeRight(typename std::vector<std::pair<Entry,Entry>>::iterator mergeBegin) {
            ASSERT(mergeBegin < _intervals.end(), "This should be called with a valid element.");
            // Find the last interval to be merged into this group
            auto mergeLast = mergeBegin;
            while (mergeLast != _intervals.end()-1 && (mergeLast+1)->first <= mergeLast->second) {
                ++mergeLast;
            }
            // Merge them
            if (mergeLast != mergeBegin) {
                mergeBegin->second = std::max(mergeBegin->second, mergeLast->second);
                _intervals.erase(mergeBegin + 1, mergeLast+1);
            }
        }

        template<class Entry>
        bool IntervalSet<Entry>::_intervalsDontOverlap() const {
            for (auto iter = _intervals.begin(); iter < _intervals.end()-1 ; ++iter) {
                if ((iter+1)->first <= iter->second) {
                    return false;
                }
            }
            return true;
        }

        template<class Entry>
        bool IntervalSet<Entry>::isCovered(Entry begin, Entry end) {
            ASSERT(begin <= end, "Invalid interval given");
            if (begin == end) {
                return true;
            }
            for (const auto &interval : _intervals) {
                if (!(begin < interval.first) && end <= interval.second) {
                    // Covered by the current interval
                    return true;
                } else if (end <= interval.first) {
                    // We're out of the region where intervals could cover us. Break early.
                    return false;
                }
            }
            // No covering interval found
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
