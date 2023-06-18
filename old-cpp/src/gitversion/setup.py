#!/usr/bin/env python3

from setuptools import setup
import versioneer

setup(name='git-version',
      version=versioneer.get_version(),
      cmdclass=versioneer.get_cmdclass(),
      description='Make git version information (e.g. git tag name, git commit id, ...) available to C++ source files.',
      author='Sebastian Messmer',
      author_email='messmer@cryfs.org',
      license='LGPLv3'
)
