# We only allow generating distribution packages if:
# - it is a release build (to ensure all generated debian packages are fine to be distributed)
# - tests are not built (because otherwise CPack would start installing googletest with the package as well)
string(TOUPPER "${CMAKE_BUILD_TYPE}" BUILDTYPE)
if(BUILDTYPE MATCHES RELEASE AND NOT BUILD_TESTING)
    if("${CMAKE_VERSION}" VERSION_LESS "3.3")
        # Earlier cmake versions generate .deb packages for which the package manager says they're bad quality
        # and asks the user whether they really want to install it. Cmake 3.3 fixes this.
        message(WARNING "Distribution package generation is only supported for CMake version >= 3.3. You're using ${CMAKE_VERSION}. You will be able to build and install CryFS, but you won't be able to generate .deb packages.")
    else("${CMAKE_VERSION}" VERSION_LESS "3.3")
        # Fix debfiles permissions. Unfortunately, git doesn't store file permissions.
        # When installing the .deb package and these files have the wrong permissions, the package manager complains.
        execute_process(COMMAND /bin/bash -c "chmod 0755 ${CMAKE_CURRENT_SOURCE_DIR}/debfiles/*")

        set(CPACK_GENERATOR TGZ DEB RPM)
        set(CPACK_PACKAGE_NAME "cryfs")
        get_git_version(GITVERSION_VERSION_STRING)
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
            set(CPACK_STRIP_FILES ON)
            set(CPACK_SOURCE_STRIP_FILES ON)
        endif(WIN32 AND NOT UNIX)
        set(CPACK_PACKAGE_EXECUTABLES "cryfs" "CryFS")
        set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
        set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
        set(CPACK_DEBIAN_PACKAGE_DEPENDS "fuse")

        set(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://www.cryfs.org")
        set(CPACK_RPM_PACKAGE_LICENSE "LGPLv3")
        set(CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postinst;${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postrm")

        include(CPack)
    endif("${CMAKE_VERSION}" VERSION_LESS "3.3")
endif(BUILDTYPE MATCHES RELEASE AND NOT BUILD_TESTING)
