# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [8.8.0](http://github.com/abdes/asap/compare/v8.7.0...v8.8.0) (2023-06-25)

### Features

- make compiler caching work for the library build ([#28](http://github.com/abdes/asap/issues/28)) ([baf2b13](http://github.com/abdes/asap/commit/baf2b132f78f0cdb3e5cf28e15e98d877ceb76e5))
- support cygwin builds ([#87](http://github.com/abdes/asap/issues/87)) ([d5d855a](http://github.com/abdes/asap/commit/d5d855a4f216ac251592cab16b7084877b4402d0))

### Bug Fixes

- [#29](http://github.com/abdes/asap/issues/29) use GNUInstallDirs variables to test for expected files ([79b8702](http://github.com/abdes/asap/commit/79b870277f20ff9ac29cedd4336a9ed37a1119ce))
- [#30](http://github.com/abdes/asap/issues/30) properly compile asm in c++ for armv7 ([9359344](http://github.com/abdes/asap/commit/93593446b3d01b9ee082f6f7065f0f3ec089146e))
- adjust project-name to not overwrite everthing ([010fa7b](http://github.com/abdes/asap/commit/010fa7b7a73065f8d3e94b8b7993277659c3f058))
- also set compiler env for the tests ([#53](http://github.com/abdes/asap/issues/53)) ([dfb8445](http://github.com/abdes/asap/commit/dfb844567a15415c784b89f12153f7bafc42566c))
- Android build for armv7-a platform ([#45](http://github.com/abdes/asap/issues/45)) ([#46](http://github.com/abdes/asap/issues/46)) ([b13d402](http://github.com/abdes/asap/commit/b13d4023cee2d8e297a7ea1c942a4f80d03553d2))
- building for arm32 without default neon flags ([#48](http://github.com/abdes/asap/issues/48)) ([13cb0fe](http://github.com/abdes/asap/commit/13cb0fe7e76410258b76975e77fd407400e3f2d1))
- check_compile_link_option should use CRYPTOPP_COMPILE_DEFINITIONS ([#37](http://github.com/abdes/asap/issues/37)) ([22a1c47](http://github.com/abdes/asap/commit/22a1c47b13ce0683a2625bca152d0470b7c17f66))
- detection and compileflag for cygwin ([7772b09](http://github.com/abdes/asap/commit/7772b09764278dcec8f1481068c1777f531d4f0a))
- detection and compileflags for mingw32 ([0b250db](http://github.com/abdes/asap/commit/0b250dba081689e856a090f660223d6c6f4c555c))
- detection of mingw64 ([fd6434b](http://github.com/abdes/asap/commit/fd6434bcfa9a2c801d3f7e1329689ffb935c988a))
- need to still include `sse_simd.cpp` in sources even when CRYPTOPP_DISABLE_ASM=1 ([0886b6e](http://github.com/abdes/asap/commit/0886b6e912a1461c0271aa617b0e7b2d2773d877))
- prepend project dir to asm source files for arm builds ([3c677c0](http://github.com/abdes/asap/commit/3c677c05288266be135197f136b5358b8a0ecb8b))
- reenable unit-tests ([c756134](http://github.com/abdes/asap/commit/c756134e1e6ab88935d297ac97f0951ef62bebb5))
- target arch detection not working on Mac ([#35](http://github.com/abdes/asap/issues/35)) ([#38](http://github.com/abdes/asap/issues/38)) ([3d8a0a5](http://github.com/abdes/asap/commit/3d8a0a576bccb718cebd4ed97a09feb520e60641))
- Typo in sources.cmake resulted in failed source deduplication ([#41](http://github.com/abdes/asap/issues/41)) ([206aab4](http://github.com/abdes/asap/commit/206aab4cb1e6e78b8460b8c0fe08449c280e321e))
- Universal macOS build failing due to unquoted architectures string ([#42](http://github.com/abdes/asap/issues/42)) ([15798ac](http://github.com/abdes/asap/commit/15798ac9c2611d5c7f9ba832e2c9159bdd8945f2))
- use correct flags for lsh512_sse.cpp ([f31a130](http://github.com/abdes/asap/commit/f31a130112a3d13014c5be381748452221a66703))
- use repos via CPM in tests consistently ([#65](http://github.com/abdes/asap/issues/65)) ([e8e5ab7](http://github.com/abdes/asap/commit/e8e5ab72918205c4b081a3214c4a8088cb055cea))
- workaround gcc-12 with -fno-devirtualize ([#25](http://github.com/abdes/asap/issues/25)) ([e7e0baa](http://github.com/abdes/asap/commit/e7e0baa12f961e227f8ab636df8f2d51495a18ca)), closes [#3](http://github.com/abdes/asap/issues/3)

## 8.7.0 (2022-09-19)

### ⚠ BREAKING CHANGES

- shared lib build is no longer allowed, until crypto++ properly supports DLL
  symbol exports and managed visibility.

### Features

- add support for cmake-presets
  ([c22d49a](http://github.com/abdes/asap/commit/c22d49a432993201b1a4ae189adef540342a58b3))
- align pkgconfig file with crypto++ project
  ([1bb96f6](http://github.com/abdes/asap/commit/1bb96f628d6a1e185db5a1b3440c37eb390d6326))
- set cmake minimum required version to 3.21
  ([31b02c4](http://github.com/abdes/asap/commit/31b02c4d1d584257e449c9d4d6fcff4913cb4a53))
- build acceleration with ccache and parallel compilation

### Bug Fixes

- [#10](http://github.com/abdes/asap/issues/10) cloning of crypto++ should be
  done in `CRYPTOPP_INCLUDE_PREFIX`
  ([feb0e11](http://github.com/abdes/asap/commit/feb0e11fdd81652d7579355783142f048fba6338))
- [#22](http://github.com/abdes/asap/issues/22) add missing neon_simd.cpp to
  sources
  ([c0f9743](http://github.com/abdes/asap/commit/c0f97430d904ead8a903f51603afaa6b1d97d003))
- [#23](http://github.com/abdes/asap/issues/23) properly detect ARM on windows
  ([6a13b4f](http://github.com/abdes/asap/commit/6a13b4fe73335e5fa768bec9d4f09fdc63603e78))
- [#24](http://github.com/abdes/asap/issues/24) apply additional compiler
  options to test sources
  ([fa5bbc1](http://github.com/abdes/asap/commit/fa5bbc1cc7cee8e71b28d773a46e158b68c4b068))
- [#4](http://github.com/abdes/asap/issues/4) build generates PIC even for
  static library
  ([476364c](http://github.com/abdes/asap/commit/476364c2a779cfaebaa236a0f886ab94d01425fb))
- [#5](http://github.com/abdes/asap/issues/5) prefix includes in build interface
  with cryptopp
  ([a6ece10](http://github.com/abdes/asap/commit/a6ece101eed109043c4dcc5a819a3541344a2c03))
- add prefix where needed and use -l to link against lib
  ([c555112](http://github.com/abdes/asap/commit/c55511285c9d4557b0f8741cbb17fe9dbaa5ef49))
- download crypto++ sources into the CURRENT cmake binary dir
  ([1c92d03](http://github.com/abdes/asap/commit/1c92d03e4963ec25d00d7d098ca7195e70889b25))
- include interface should work when package is used via find_package
  ([eaf4ca0](http://github.com/abdes/asap/commit/eaf4ca0d3540f94e28b26e7bff561e1451420a5c))
- install header files under cryptopp folder
  ([5d72716](http://github.com/abdes/asap/commit/5d727164019790aa1d0c8e2301dfdcfef387a644))
- only detect `arm` as ARM32 if it's not a substring
  ([4f52cc6](http://github.com/abdes/asap/commit/4f52cc63b6e0bdd2a02a18745ba8984bcdad79e6))
- prefix includes in install interface with cryptopp
  ([2dbdc8a](http://github.com/abdes/asap/commit/2dbdc8a2dfc587967ed83d6b62e22fb17400fb52))
- rollback [#4](http://github.com/abdes/asap/issues/4)
  ([26f786e](http://github.com/abdes/asap/commit/26f786e98212f0e8c952df55a22d132458b5f78b))
- set MSVC specific options within if(MSVC) block
  ([add97fb](http://github.com/abdes/asap/commit/add97fb07b43767913be0c3a9ce4a45d3ced9dbd))
- wrong '-D' when adding to COMPILE_DEFINITIONS
  ([1561116](http://github.com/abdes/asap/commit/1561116182906a160d342b0446798e78fff486fc))
- wrong linker flags generated in pkgconfig.pc
  ([7252030](http://github.com/abdes/asap/commit/7252030e64410710bdbcdb2b921d236c2c6c960f))
- wrong test when setting CRYPTOPP_PROJECT_DIR
  ([22016ce](http://github.com/abdes/asap/commit/22016cebdbe7f204b2072dee76bdab185a6f59a3))
- cleanup library targets to align with cmake best practices
  ([b4e850b](http://github.com/abdes/asap/commit/b4e850b112593073bcca64d1f88d2192926c05d5))
- many more fixes and tweaks...

### Documentation

- add a `before you ask` section
  ([f071c14](http://github.com/abdes/asap/commit/f071c14eb7c1a2813f6a949c5ed99bed0c0410d3))
- add link to change of repo announcement
  ([63f4e10](http://github.com/abdes/asap/commit/63f4e10af10c8c51dc53827e430cec89748db6e4))
- small corrections to contribution guide
  ([439b1aa](http://github.com/abdes/asap/commit/439b1aa357290784881a7af3083dd63de873bc23))
- update project intro and new direction
  ([234aac7](http://github.com/abdes/asap/commit/234aac76e6b51a66e9e6c6fc3fd07f236814d2c5))
- update to refer to the new link target names
  ([6cab962](http://github.com/abdes/asap/commit/6cab962e7cbbac533da1dde9023de185a9ef0238))

## From original repo

### Features

- cache results of check compile/link operations
  ([8e92fe0](http://github.com/abdes/asap/commit/8e92fe0544755d34ec569d5f561f62c419fa42dc))
- flexible fetching of crypto++ sources
  ([82a137e](http://github.com/abdes/asap/commit/82a137ed6696fe48ddcd704f65710a7588f3b8a4))

### Bug Fixes

- [#88](http://github.com/abdes/asap/issues/88)
  [#90](http://github.com/abdes/asap/issues/90)
  [#91](http://github.com/abdes/asap/issues/91)
  [#92](http://github.com/abdes/asap/issues/92)
  [#93](http://github.com/abdes/asap/issues/93)
  ([6833a02](http://github.com/abdes/asap/commit/6833a024644f742fb5ecda24841708843327eccb))
- [#93](http://github.com/abdes/asap/issues/93) properly set compiler options
  ([8fb2f02](http://github.com/abdes/asap/commit/8fb2f02377307f6ab86191b56d036cf7d85d31f0))
- [#97](http://github.com/abdes/asap/issues/97) cryptest runs in wrong directory
  when in sub-project
  ([dea86d4](http://github.com/abdes/asap/commit/dea86d4316ae85cfbe18b521edf56af92c22d07f))
- [#98](http://github.com/abdes/asap/issues/98) standard-cpm test case not
  running properly when in a sub-project
  ([96a78f7](http://github.com/abdes/asap/commit/96a78f741bda12a7730ca3943ace4c24070dd431))
- adjust header files path for install
  ([e28a9b1](http://github.com/abdes/asap/commit/e28a9b133f0aea370b317feb1ed7feafc7311752))
- duplicate cryptest.exe target
  ([2c39b76](http://github.com/abdes/asap/commit/2c39b764bf88ffb4029becdf6e04208c79bc9c98))
- missing compiler definitions for c++ files
  ([8e6ed7f](http://github.com/abdes/asap/commit/8e6ed7f95b1db2e0806e1e55aafdd25d41a3a196))
- reorganize options and disable tests when CRYPTOPP_BUILD_TESTING is OFF
  ([#96](http://github.com/abdes/asap/issues/96))
  ([cbed9fb](http://github.com/abdes/asap/commit/cbed9fb37ed698a09bca5174cbdf5e2d5d428b6f))
- use correct lib in linker options
  ([184f30b](http://github.com/abdes/asap/commit/184f30b56bcbca24582ae374a50b0b28a517c4f4))

### Documentation

- add instruction for conventional commits and changelog
  ([26194ff](http://github.com/abdes/asap/commit/26194ff3d1db5c50fd1c81ed59ec0115eb26b73c))
- add warning regarding version format in tags
  ([96f73bb](http://github.com/abdes/asap/commit/96f73bb12777e4679c12c22e761f6e95e6a9abfd))
- update after refactoring
  ([ff63aa4](http://github.com/abdes/asap/commit/ff63aa4694334161c922ab0f8d434340d2993644))
