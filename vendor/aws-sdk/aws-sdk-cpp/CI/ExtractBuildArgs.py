#!/usr/bin/python
import sys
import json

if len(sys.argv) != 2:
    print >> sys.stderr, "    Usage: python ExtractBuildArgs.py <ArgName>"
    exit (-1)

try:
    data = json.load(open('BuildSpec.json'))
    if sys.argv[1] == "cmakeFlags" and data["cmakeFlags"] != "":
        print(data["cmakeFlags"])
    elif sys.argv[1] == "branch" and data["branch"] != "":
        print(data["branch"])
except:
    print >> sys.stderr, "No related args found in BuildSpec.json"
    exit(-1)
