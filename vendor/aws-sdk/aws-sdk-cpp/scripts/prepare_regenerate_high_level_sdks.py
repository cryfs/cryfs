#!/usr/bin/env python

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
import argparse
import re
import os
import sys
import filecmp
import fnmatch

highLevelSdkList = [
    "access-management", 
    "identity-management", 
    "queues", 
    "transfer", 
    "s3-encryption", 
    "text-to-speech"
];

def ParseArguments():
    argMap = {}

    parser = argparse.ArgumentParser(description="Prepare for high level sdks' regeneration")
    parser.add_argument("--highLevelSdkName", action="store")
    args = vars( parser.parse_args() )
    argMap[ "highLevelSdkName" ] = args[ "highLevelSdkName" ] or None
    return argMap

def prepareAutopkg(highLevelSdkName):
    autopkgFile = "aws-cpp-sdk-" + highLevelSdkName + "/nuget/" + "aws-cpp-sdk-" + highLevelSdkName + ".autopkg";
    with open(autopkgFile, "rt") as ftarget:
        content = ftarget.read()

    """
    The following regex code is going to change content like:
        version : 1.0.153;
        dependencies {
            packages: {
                AWSSDKCPP-Core/1.0.140,
                AWSSDKCPP-S3-Encryption/1.0.20060301.142
                AWSSDKCPP-sqs/2.3.20070319.141
            }
        }
    to:
        version : @RUNTIME_MAJOR_VERSION@.@RUNTIME_MINOR_VERSION@;
        dependencies {
            packages: {
                AWSSDKCPP-Core/@RUNTIME_MAJOR_VERSION@.@RUNTIME_MINOR_VERSION@,
                AWSSDKCPP-S3-Encryption/@RUNTIME_MAJOR_VERSION@.20060301.@RUNTIME_MINOR_VERSION@
                AWSSDKCPP-sqs/@RUNTIME_MAJOR_VERSION@.20070319.@RUNTIME_MINOR_VERSION@
            }
        }
    note:
        RUNTIME_MAJOR_VERSION has two parts separated by '.', like 1.0, 2.1 and so on.
        RUNTIME_MINOR_VERSION is a single digit string like 79, 150, 142 and so on.
        AWSSDKCPP-Core dosen't have a API version string in between MAJOR and MINOR version strings.
        These version releated strings are changed to special tokens so as to be replaced with actual versions during release stage in our code pipeline.
    """
    newContent = re.sub(r"version : \d+\.\d+\.\d+;", "version : @RUNTIME_MAJOR_VERSION@.@RUNTIME_MINOR_VERSION@;", content);
    newContent = re.sub(r"AWSSDKCPP-Core/[^,]+?(,{0,1})\n", r"AWSSDKCPP-Core/@RUNTIME_MAJOR_VERSION@.@RUNTIME_MINOR_VERSION@\1\n", newContent);
    newContent = re.sub(r"(AWSSDKCPP-[a-zA-Z\-\d]+)/\d+\.\d+\.(\d+)[^,]{0,}?(,{0,1})\n", r"\1/@RUNTIME_MAJOR_VERSION@.\2.@RUNTIME_MINOR_VERSION@\3\n", newContent);

    if (content == newContent):
        return False;

    with open(autopkgFile, "wt") as fdest:
        fdest.write(newContent)
    return

def Main():
    arguments = ParseArguments()

    if arguments['highLevelSdkName']:
        print('Preparing {}.'.format(arguments['highLevelSdkName']))
        prepareAutopkg(arguments['highLevelSdkName']);
    else:
        for svc in highLevelSdkList:
            prepareAutopkg(svc);

Main()

