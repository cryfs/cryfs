#!/usr/bin/python

import argparse
import shutil
import subprocess
import re
import subprocess
import os
import zipfile
import io
import json

def Main():

    parser = argparse.ArgumentParser(description="Creates a release doc based on a list of changes.")
    parser.add_argument("--changesList", action="store")

    args = vars( parser.parse_args() )

    changes = args["changesList"]

    changeDoc = {}

    changeList = changes.split()
    releases = []
    release = {}
    features = []

    for change in changeList:
        feature = {}
        feature["service-name"] = change.replace("aws-cpp-sdk-", "")
        features.append(feature)
        
    release["features"] = features
    releases.append(release)
    changeDoc["releases"] = releases

    print(json.dumps(changeDoc))

Main()
