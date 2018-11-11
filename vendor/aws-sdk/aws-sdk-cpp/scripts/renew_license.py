#
# Copyright 2010-2017 Amazon.com, Inc. or its affiliates. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License").
# You may not use this file except in compliance with the License.
# A copy of the License is located at
#
#  http://aws.amazon.com/apache2.0
#
# or in the "license" file accompanying this file. This file is distributed
# on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing
# permissions and limitations under the License.
#

# This script is used to update License end year for those hand crafted files
# Auto-generated files will be automatically updated by code-generator when published to release candidate in our code pipeline
# Simply modify OldLicense and NewLicense before running this script

import fnmatch
import filecmp
import os
import sys
import datetime
import re

Now = datetime.datetime.now()

NewLicense = "Copyright 2010-" + str(Now.year) + " Amazon.com"

def updateLicense(inputFile):
    with open(inputFile) as ftarget:
        content = ftarget.read()

    newContent = re.sub(r"Copyright 2010-201[\d] Amazon.com", NewLicense, content); 
    if (content == newContent):
        return False;

    with open(inputFile, "w") as fdest:
        fdest.write(newContent)
    return True;

RootDir = os.path.dirname(os.path.dirname(os.path.realpath(__file__)));
for root, dirnames, filenames in os.walk(RootDir):
    for filename in fnmatch.filter(filenames, '*'):
        targetFile = os.path.join(root, filename);
        ret = updateLicense(targetFile)
