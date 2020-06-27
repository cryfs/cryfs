from conans import ConanFile, CMake

class CryFSConan(ConanFile):
	settings = "os", "compiler", "build_type", "arch"
	requires = [
		"range-v3/0.9.1@ericniebler/stable",
		"spdlog/1.4.2",
	]
	generators = "cmake"
	default_options = {
		# Need to disable boost-math because that doesn't compile for some reason on CI
		"boost:without_math": True,
		"boost:without_wave": True,
		"boost:without_container": True,
		"boost:without_exception": True,
		"boost:without_graph": True,
		"boost:without_iostreams": False,
		"boost:without_locale": True,
		"boost:without_log": True,
		"boost:without_random": True,
		"boost:without_regex": True,
		"boost:without_mpi": True,
		"boost:without_serialization": True,
		"boost:without_coroutine": True,
		"boost:without_fiber": True,
		"boost:without_context": True,
		"boost:without_timer": True,
		"boost:without_date_time": True,
		"boost:without_atomic": True,
		"boost:without_graph_parallel": True,
		"boost:without_python": True,
		"boost:without_test": True,
		"boost:without_type_erasure": True,
	}

	def requirements(self):
		if self.settings.os == "Windows":
			self.requires("boost/1.69.0@conan/stable")
		else:
			self.requires("boost/1.65.1@conan/stable")
