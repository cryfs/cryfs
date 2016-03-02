#!/usr/bin/env python

from setuptools import setup
import versioneer

setup(name='git-version',
      version=versioneer.get_version(),
      cmdclass=versioneer.get_cmdclass(),
      description='Make git version information (e.g. git tag name, git commit id, ...) available to your C++ or python source files. A simple use case scenario is to output this version information when the application is called with "--version".',
      author='Sebastian Messmer',
      author_email='messmer@cryfs.org',
      license='GPLv3',
      packages=['gitversion'],
      entry_points = {
        'console_scripts': [
          "git-version = __main__:main"
        ]
      },
)
