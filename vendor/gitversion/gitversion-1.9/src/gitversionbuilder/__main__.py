#!/usr/bin/env python

import argparse
import importlib
from gitversionbuilder import main


try:
    Version = importlib.import_module(".Version", package="gitversionbuilder")
except ImportError:
    Version = importlib.import_module(".DummyVersion", package="gitversionbuilder")


def run_main():
    parser = argparse.ArgumentParser(description="Create a source file containing git version information.")
    parser.add_argument('--version', action='version', version=Version.VERSION_STRING)
    parser.add_argument('--lang', choices=['cpp', 'python'], required=True)
    parser.add_argument('--dir', default='.')
    parser.add_argument('file')
    args = parser.parse_args()

    print("Creating git version information from %s" % args.dir)

    main.create_version_file(git_directory=args.dir, output_file=args.file, lang=args.lang)


if __name__ == '__main__':
    run_main()
