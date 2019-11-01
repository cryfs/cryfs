from conans import ConanFile, CMake

class CryFSConan(ConanFile):
	settings = "os", "compiler", "build_type", "arch"
	requires = [
		"range-v3/0.9.1@ericniebler/stable",
	]
	generators = "cmake"
	default_options = {
		# Need to disable boost-math because that doesn't compile for some reason on CI
		"boost:without_math": True,
	}

	def requirements(self):
		if self.settings.os == "Windows":
			self.requires("boost/1.69.0@conan/stable")
		else:
			self.requires("boost/1.65.1@conan/stable")
