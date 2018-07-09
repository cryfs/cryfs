// This is a small executable that prints its first argument and exits with the exit status in its second argument

#include <iostream>

int main(int argc, char* argv[]) {
	if (argc != 3) {
		std::cerr << "Wrong number of arguments" << std::endl;
		abort();
	}

	std::cout << argv[1];

	int exit_status = std::atoi(argv[2]);
	return exit_status;
}
