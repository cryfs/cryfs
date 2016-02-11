include (../vendor/gitversion/gitversion-1.8/cmake.cmake)

# Fix debfiles permissions. Unfortunately, git doesn't store file permissions.
# When installing the .deb package and these files have the wrong permissions, the package manager complains.
execute_process(COMMAND /bin/bash -c "chmod 0755 ${CMAKE_CURRENT_SOURCE_DIR}/debfiles/*")

set(CPACK_GENERATOR TGZ DEB RPM)
set(CPACK_PACKAGE_NAME "cryfs")
set(CPACK_PACKAGE_VERSION "${GITVERSION_VERSION_STRING}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Encrypt your files and store them in the cloud.")
set(CPACK_PACKAGE_DESCRIPTION "CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.")
set(CPACK_PACKAGE_CONTACT "Sebastian Messmer <messmer@cryfs.org>")
set(CPACK_PACKAGE_VENDOR "Sebastian Messmer")
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE")
set(CPACK_PACKAGE_INSTALL_DIRECTORY "CMake ${CMake_VERSION_MAJOR}.${CMake_VERSION_MINOR}")
if(WIN32 AND NOT UNIX)
    # There is a bug in NSI that does not handle full unix paths properly. Make
    # sure there is at least one set of four (4) backlasshes.
    #set(CPACK_PACKAGE_ICON "${CMake_SOURCE_DIR}/Utilities/Release\\\\InstallIcon.bmp")
    #set(CPACK_NSIS_INSTALLED_ICON_NAME "bin\\\\cryfs.exe")
    #set(CPACK_NSIS_DISPLAY_NAME "${CPACK_PACKAGE_INSTALL_DIRECTORY} CryFS")
    #set(CPACK_NSIS_HELP_LINK "http:\\\\\\\\www.cryfs.org")
    #set(CPACK_NSIS_URL_INFO_ABOUT "http:\\\\\\\\www.cryfs.org")
    #set(CPACK_NSIS_CONTACT "messmer@cryfs.org")
    #set(CPACK_NSIS_MODIFY_PATH ON)
else(WIN32 AND NOT UNIX)
    set(CPACK_STRIP_FILES "bin/cryfs")
    set(CPACK_SOURCE_STRIP_FILES "")
endif(WIN32 AND NOT UNIX)
set(CPACK_PACKAGE_EXECUTABLES "cryfs" "CryFS")
set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
set(CPACK_DEBIAN_PACKAGE_RECOMMENDS "fuse")

set(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://www.cryfs.org")
set(CPACK_RPM_PACKAGE_LICENSE "LGPLv3")
set(CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postinst;${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postrm")

include(CPack)