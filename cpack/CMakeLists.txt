# appends a build number from the APPVEYOR_BUILD_NUMBER environment variable as fourth component to a version number,
# i.e. "0.10" becomes "0.10.0.[buildnumber]", "1" becomes "1.0.0.[buildnumber]".
function(append_build_number VERSION_NUMBER OUTPUT)
	string(REPLACE "." ";" VERSION_COMPONENTS ${STRIPPED_VERSION_NUMBER})
	list(LENGTH VERSION_COMPONENTS NUM_VERSION_COMPONENTS)
	if (${NUM_VERSION_COMPONENTS} LESS_EQUAL 0)
		message(FATAL_ERROR "Didn't find any version components")
	endif()
	if (${NUM_VERSION_COMPONENTS} LESS_EQUAL 1)
		string(APPEND STRIPPED_VERSION_NUMBER ".0")
	endif()
	if (${NUM_VERSION_COMPONENTS} LESS_EQUAL 2)
		string(APPEND STRIPPED_VERSION_NUMBER ".0")
	endif()
	if (NOT $ENV{APPVEYOR_BUILD_NUMBER} STREQUAL "")
		string(APPEND STRIPPED_VERSION_NUMBER ".$ENV{APPVEYOR_BUILD_NUMBER}")
	endif()
	set(${OUTPUT} "${STRIPPED_VERSION_NUMBER}" PARENT_SCOPE)
endfunction()

if("${CMAKE_VERSION}" VERSION_LESS "3.3")
    # Earlier cmake versions generate .deb packages for which the package manager says they're bad quality
    # and asks the user whether they really want to install it. Cmake 3.3 fixes this.
    message(WARNING "Distribution package generation is only supported for CMake version >= 3.3. You're using ${CMAKE_VERSION}. You will be able to build and install CryFS, but you won't be able to generate .deb packages.")
else()
    # Fix debfiles permissions. Unfortunately, git doesn't store file permissions.
    # When installing the .deb package and these files have the wrong permissions, the package manager complains.
    execute_process(COMMAND /bin/bash -c "chmod 0755 ${CMAKE_CURRENT_SOURCE_DIR}/debfiles/*")

    set(CPACK_PACKAGE_NAME "cryfs")
    set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Encrypt your files and store them in the cloud.")
    set(CPACK_PACKAGE_DESCRIPTION "CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.")
    set(CPACK_PACKAGE_CONTACT "Sebastian Messmer <messmer@cryfs.org>")
    set(CPACK_PACKAGE_VENDOR "Sebastian Messmer")
    set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE.txt")

    get_git_version(GITVERSION_VERSION_STRING)

    if(WIN32 AND NOT UNIX)
        set(CPACK_GENERATOR WIX)

        string(REGEX REPLACE "^([0-9\\.]+)([-+][0-9\\.a-zA-Z+-]+)?$" "\\1" STRIPPED_VERSION_NUMBER "${GITVERSION_VERSION_STRING}")
        append_build_number(${STRIPPED_VERSION_NUMBER} WIX_VERSION_NUMBER)
        message(STATUS "WIX package version is ${WIX_VERSION_NUMBER}")
        set(CPACK_PACKAGE_VERSION "${WIX_VERSION_NUMBER}")

        set(CPACK_WIX_UPGRADE_GUID "8b872ce1-557d-48e6-ac57-9f5e574feabf")
        set(CPACK_WIX_PRODUCT_GUID "26116061-4f99-4c44-a178-2153fa396308")
        #set(CPACK_WIX_PRODUCT_ICON "...")
        set(CPACK_WIX_PROPERTY_ARPURLINFOABOUT "https://www.cryfs.org")
        set(CPACK_PACKAGE_INSTALL_DIRECTORY "CryFS/${GITVERSION_VERSION_STRING}")
        set(CPACK_WIX_PATCH_FILE "${CMAKE_CURRENT_SOURCE_DIR}/wix/change_path_env.xml")
    else()
        set(CPACK_GENERATOR TGZ DEB RPM)
        set(CPACK_PACKAGE_VERSION "${GITVERSION_VERSION_STRING}")
        set(CPACK_STRIP_FILES OFF)
        set(CPACK_SOURCE_STRIP_FILES OFF)
    endif()
    set(CPACK_PACKAGE_EXECUTABLES "cryfs" "CryFS")
    set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
    set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
    # Needs gnupg2, lsb-release for postinst script
    set(CPACK_DEBIAN_PACKAGE_DEPENDS "fuse, gnupg2, lsb-release")

    set(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://www.cryfs.org")
    set(CPACK_RPM_PACKAGE_LICENSE "LGPLv3")
    set(CPACK_RPM_PACKAGE_DESCRIPTION ${CPACK_PACKAGE_DESCRIPTION})
    set(CPACK_RPM_EXCLUDE_FROM_AUTO_FILELIST_ADDITION "/usr/bin;/usr/share/man;/usr/share/man/man1")
    set(CPACK_DEBIAN_PACKAGE_CONTROL_EXTRA "${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postinst;${CMAKE_CURRENT_SOURCE_DIR}/debfiles/postrm")

    include(CPack)
endif()
