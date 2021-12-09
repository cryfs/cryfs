// This is a small executable that exits with the exit status in its first argument and before exiting prints all other arguments, each on a separate line.

#include <iostream>
#include <cstdlib>

int main(int argc, char *argv[])
{
	if (argc < 2)
	{
		std::cerr << "Wrong number of arguments" << std::endl;
		std::abort();
	}

	for (int i = 2; i < argc; ++i)
	{
		std::cout << argv[i] << "\n";
	}

	int exit_status = static_cast<int>(std::strtol(argv[1], nullptr, 10));
	return exit_status;
}
