// Workaround for https://stackoverflow.com/questions/77444892/xcode-15-0-1-macos-sonoma-clang-archive-or-linking-issue#comment137030259_77630733
// Seems every library must have at least one symbol or it won't link correctly on macos
// Error message was: "ld: archive member '/' not a mach-o file in '/Users/runner/work/cryfs/cryfs/build/src/fspp/fs_interface/libfspp-interface.a'"

namespace fspp
{
    // NOLINTNEXTLINE(misc-use-internal-linkage) - this function exists precisely to give
    // the library an external symbol, per the comment above. Internal linkage would emit
    // no symbol at all and bring back the macOS link error this file works around.
    void dummy() {}
}
