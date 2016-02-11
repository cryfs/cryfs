from tempfile import NamedTemporaryFile, mkdtemp
import os
import shutil
import subprocess
import random
import string
from gitversionbuilder.utils import ChDir


class CodeAsserts(object):
    def assertCodeEqual(self, expected, actual):
        self.assertEqual(self._normalize(expected), self._normalize(actual))

    def read_file(self, filename):
        with open(filename, 'r') as open_file:
            return open_file.read()

    def _normalize(self, string):
        lines = string.splitlines()
        normalized_lines = map(self._normalize_line, lines)
        without_empty_lines = filter(None, normalized_lines)
        return "\n".join(without_empty_lines)

    def _normalize_line(self, line):
        tokens = line.split()
        return " ".join(tokens)


class TempFile(object):
    def __enter__(self):
        f = NamedTemporaryFile()
        f.close() # This also deletes the file
        self.filename = f.name
        return f.name

    def __exit__(self, type, value, tb):
        if os.path.isfile(self.filename):
            os.remove(self.filename)


class GitDir(object):
    def __enter__(self):
        self.dir = mkdtemp()
        self._setup_git()
        return self

    def __exit__(self, type, value, traceback):
        shutil.rmtree(self.dir)

    def _setup_git(self):
        with ChDir(self.dir):
            self._silent_call(["git", "init"])
            self._silent_call(["git", "config", "user.email", "you@example.com"]);
            self._silent_call(["git", "config", "user.name", "Your Name"]);

    def create_git_commit(self):
        self.add_tracked_file()
        with ChDir(self.dir):
            self._silent_call(["git", "commit", "-m", "message"])
            commit_id = self._silent_call(["git", "rev-parse", "--short", "HEAD"]).strip()
            return commit_id

    def add_untracked_file(self):
        filename = self._random_string(10)
        with ChDir(self.dir):
            self._silent_call(["touch", filename])
            return filename

    def add_tracked_file(self):
        filename = self.add_untracked_file()
        with ChDir(self.dir):
            self._silent_call(["git", "add", filename])
            return filename

    def modify_file(self, filename):
        content = self._random_string(10)
        with ChDir(self.dir):
            with open(filename, 'w') as file:
                file.write(content)

    def _random_string(self, length):
        return ''.join(random.choice(string.ascii_lowercase) for _ in range(length))

    def _silent_call(self, command):
        with open(os.devnull, 'w') as devnull:
            return subprocess.check_output(command, stderr=devnull).decode()

    def create_git_branch(self, branch_name):
        with ChDir(self.dir):
            self._silent_call(["git", "checkout", "-b", branch_name])

    def switch_git_branch(self, branch_name):
        with ChDir(self.dir):
            self._silent_call(["git", "checkout", branch_name])

    def checkout_git_commit(self, commit_id):
        with ChDir(self.dir):
            self._silent_call(["git", "checkout", commit_id])

    def create_git_tag(self, tag_name):
        with ChDir(self.dir):
            self._silent_call(["git", "tag", tag_name])
