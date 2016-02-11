#!/usr/bin/env python

from setuptools import setup
from gitversionbuilder import main

main.create_version_file(git_directory=".", output_file="gitversionbuilder/Version.py", lang="python")
version = main.get_version(git_directory=".")

setup(name='git-version',
      version=version.version_string,
      description='Make git version information (e.g. git tag name, git commit id, ...) available to your C++ or python source files. A simple use case scenario is to output this version information when the application is called with "--version".',
      author='Sebastian Messmer',
      author_email='heinzisoft@web.de',
      license='GPLv3',
      url='https://github.com/smessmer/gitversion',
      packages=['gitversionbuilder'],
      tests_require=['tempdir'],
      test_suite='test',
      entry_points = {
        'console_scripts': [
          "git-version = gitversionbuilder.__main__:run_main"
        ]
      },
      classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Environment :: Console",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: GNU General Public License v3 (GPLv3)",
        "Programming Language :: Python",
        "Programming Language :: C++",
        "Topic :: Software Development :: Build Tools",
        "Topic :: Software Development :: Code Generators",
        "Topic :: Software Development :: Version Control"
      ]
      )
