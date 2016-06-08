#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_STAT_H
#define MESSMER_CPPUTILS_SYSTEM_STAT_H

/**
 * For platform independence: Apple doesn't have stat.st_atim, but stat.st_atimespec
 */
#ifdef __APPLE__
# define st_atim st_atimespec
# define st_mtim st_mtimespec
# define st_ctim st_ctimespec
#endif

#endif
