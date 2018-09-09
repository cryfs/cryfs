project (doc)

IF (WIN32)
	MESSAGE(STATUS "This is Windows. Will not install man page")
ELSE (WIN32)
	INCLUDE(GNUInstallDirs)

	find_program(GZIP gzip)

	add_custom_command(
	  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/cryfs.1.gz
	  COMMAND ${GZIP} -c ${CMAKE_CURRENT_SOURCE_DIR}/man/cryfs.1 > ${CMAKE_CURRENT_BINARY_DIR}/cryfs.1.gz
	)
	add_custom_target(man ALL DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/cryfs.1.gz)

	install(FILES ${CMAKE_CURRENT_BINARY_DIR}/cryfs.1.gz
			DESTINATION ${CMAKE_INSTALL_MANDIR}/man1
			CONFIGURATIONS Release
	)
ENDIF(WIN32)
