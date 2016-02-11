# Earlier cmake versions generate .deb packages for which the package manager says they're bad quality
# and asks the user whether they really want to install it. Cmake 3.3 fixes this.
cmake_minimum_required(VERSION 3.3)

include(utils.cmake)

require_gcc_version(4.8)

add_subdirectory(vendor)
add_subdirectory(gitversion)
add_subdirectory(src)
add_subdirectory(test)

# Fix debfiles permissions. Unfortunately, git doesn't store file permissions.
# When installing the .deb package and these files have the wrong permissions, the package manager complains.
EXECUTE_PROCESS(COMMAND /bin/bash -c "chmod 0755 ${CMAKE_CURRENT_SOURCE_DIR}/debfiles/*")

INSTALL(TARGETS ${BII_src_main_TARGET}
        DESTINATION bin
        CONFIGURATIONS Release)

SET(CPACK_GENERATOR TGZ DEB RPM)
SET(CPACK_PACKAGE_NAME "cryfs")
SET(CPACK_PACKAGE_VERSION "${GITVERSION_VERSION_STRING}")
SET(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Encrypt your files and store them in the cloud.")
SET(CPACK_PACKAGE_DESCRIPTION "CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.")
SET(CPACK_PACKAGE_CONTACT "Sebastian Messmer <messmer@cryfs.org>")
SET(CPACK_PACKAGE_VENDOR "Sebastian Messmer")
SET(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/LICENSE")
SET(CPACK_PACKAGE_INSTALL_DIRECTORY "CMake ${CMake_VERSION_MAJOR}.${CMake_VERSION_MINOR}")
IF(WIN32 AND NOT UNIX)
        # There is a bug in NSI that does not handle full unix paths properly. Make
        # sure there is at least one set of four (4) backlasshes.
        #SET(CPACK_PACKAGE_ICON "${CMake_SOURCE_DIR}/Utilities/Release\\\\InstallIcon.bmp")
        #SET(CPACK_NSIS_INSTALLED_ICON_NAME "bin\\\\cryfs.exe")
        #SET(CPACK_NSIS_DISPLAY_NAME "${CPACK_PACKAGE_INSTALL_DIRECTORY} CryFS")
        #SET(CPACK_NSIS_HELP_LINK "http:\\\\\\\\www.cryfs.org")
        #SET(CPACK_NSIS_URL_INFO_ABOUT "http:\\\\\\\\www.cryfs.org")
        #SET(CPACK_NSIS_CONTACT "messmer@cryfs.org")
        #SET(CPACK_NSIS_MODIFY_PATH ON)
ELSE(WIN32 AND NOT UNIX)
        SET(CPACK_STRIP_FILES "bin/cryfs")
        SET(CPACK_SOURCE_STRIP_FILES "")
ENDIF(WIN32 AND NOT UNIX)
SET(CPACK_PACKAGE_EXECUTABLES "cryfs" "CryFS")
SET(CPACK_DEBIAN_PACKAGE_SECTION "utils")
SET(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
SET(CPACK_DEBIAN_PACKAGE_RECOMMENDS "fuse")

SET(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://www.cryfs.org")
SET(CPACK_RPM_PACKAGE_LICENSE "LGPLv3")
SET(CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postinst;${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postrm")

INCLUDE(CPack)

