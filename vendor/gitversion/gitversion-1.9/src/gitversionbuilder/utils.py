import os
import sys

# Use this like
# > with ChDir(my_dir):
# >   do_something()
# Then, the working directory will be set to my_dir, do_something() will be called,
# and the working directory will be set back.
class ChDir(object):
    def __init__(self, dir):
        self.dir = dir

    def __enter__(self):
        self.old_dir = os.getcwd()
        os.chdir(self.dir)

    def __exit__(self, type, value, traceback):
        os.chdir(self.old_dir)


class EqualityMixin(object):
    def __eq__(self, other):
        return (isinstance(other, self.__class__)
                and self.__dict__ == other.__dict__)

    def __ne__(self, other):
        return not self.__eq__(other)


IS_PYTHON_2 = sys.version_info[0] == 2
IS_PYTHON_3 = sys.version_info[0] == 3


def isstring(obj):
    if IS_PYTHON_2:
        return isinstance(obj, basestring)
    else:
        return isinstance(obj, str)
