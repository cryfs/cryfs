import unittest
import os
from gitversionbuilder import utils


class UtilsTest(unittest.TestCase):
    def test_chdir_to_root(self):
        curdir = os.getcwd()
        with utils.ChDir('/'):
            self.assertEqual('/', os.getcwd())
        self.assertEqual(curdir, os.getcwd())

    def test_chdir_to_parent(self):
        curdir = os.getcwd()
        with utils.ChDir('..'):
            self.assertEqual(os.path.abspath(os.path.join(curdir, '..')), os.getcwd())
        self.assertEqual(curdir, os.getcwd())


if __name__ == '__main__':
    unittest.main()
