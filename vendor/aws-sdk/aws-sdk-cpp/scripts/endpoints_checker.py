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

# This script is used to check aws-cpp-sdk source file to identify hard coded endpoints in source code.
# This identification action is corresponding to a COE https://coe.amazon.com/coes/70203.
# Complete endpoints list can be found at http://docs.aws.amazon.com/general/latest/gr/rande.html
# Some appearances of endpoints in source files are intentional based on api description such as files' name end with [svc]Endpoint.cpp [svc]/model/*Region.cpp and etc.
# These files can be white listed during checking by adding skip pattern in below SkipFile function.
# Files will be comments-stripped before checking to avoid false alarm.
# If identified, file name, the first appearance of hard coded endpoints and context will be output to command-line.
# The exit code will be 1 if identified any file with hard coded endpoints, 0 otherwise.

import os
import re

"""
endpoints = ["us-east-1", "us-east-2", 
          "us-west-1", "us-west-2", 
          "eu-west-1", "eu-west-2", "eu-west-3", "eu-central-1", 
          "ap-southeast-1", "ap-southeast-2", "ap-northeast-1", "ap-northeast-2", "ap-south-1",
          "sa-east-1", 
          "cn-north-1", "cn-northwest-1",
          "ca-central-1",
          "us-gov-west-1"];
"""

def RemoveCPPComments(text):
    def replacer(match):
        s = match.group(0);
        if s.startswith('/'):
            return " "; # int/**/x=5 -> int x=5, instead of intx=5.
        else:
            return s;
    pattern = re.compile(r'//.*?$|/\*.*?\*/|"(?:\\.|[^\\"])*"', re.DOTALL | re.MULTILINE);
    return re.sub(pattern, replacer, text);

def SkipFile(fileName):
    skipFilePattern = re.compile(r'.*source\/model\/BucketLocationConstraint\.cpp'
            '|.*source\/model\/.*Region.*\.cpp'
            '|.*source\/[^\/]+Endpoint\.cpp'
            '|.*aws-cpp-sdk-core\/include\/aws\/core/\Region.h'
            '|.*tests\/.*Test\.cpp'
            # add more white lists here
            );
    if skipFilePattern.match(fileName):
        return True;
    return False;

def ScanContent(content):
    EndpointsPattern = re.compile(r'us-east-1|us-east-2|us-west-1|us-west-2|eu-west-1|eu-west-2|eu-west-3|eu-central-1|ap-southeast-1|ap-southeast-2|ap-northeast-1|ap-northeast-2|ap-south-1|sa-east-1|cn-north-1|cn-northwest-1|ca-central-1|us-gov-west-1');
    return re.search(EndpointsPattern, content);

def CheckFile(inputFile):
    if SkipFile(inputFile):
        return False;

    with open(inputFile) as ftarget:
        content = ftarget.read();

    strippedContent = RemoveCPPComments(content);
    match = ScanContent(strippedContent);
    if match:
        print inputFile;
        print "..." + strippedContent[match.start() : match.end()] + "...";
        return True;

    return False;

###################Test Start#####################################
assert RemoveCPPComments("") == "";
assert RemoveCPPComments("/") == "/";
assert RemoveCPPComments("//") == " ";
assert RemoveCPPComments("abc//test") == "abc ";
assert RemoveCPPComments("//test") == " ";
assert RemoveCPPComments("abc") == "abc";
assert RemoveCPPComments("/abc") == "/abc";
assert RemoveCPPComments("/abc/") == "/abc/";
assert RemoveCPPComments("/**/") == " ";
assert RemoveCPPComments("/*") == "/*";
assert RemoveCPPComments("*/") == "*/";
assert RemoveCPPComments("/*/") == "/*/";
assert RemoveCPPComments("\"") == "\"";
assert RemoveCPPComments(r'"Hello \"/*test*/World\""') == r'"Hello \"/*test*/World\""';
assert RemoveCPPComments("/*abc*/") == " ";
assert RemoveCPPComments(r'abc="//"//comments') == r'abc="//" ';
assert RemoveCPPComments(r'abc="/*inner comments*/"/*\
        multiline\
        comments*/') == r'abc="/*inner comments*/" ';

assert SkipFile("source/model/Regionabc.cpp") == True;
assert SkipFile("source/model/abcRegion.cpp") == True;
assert SkipFile("source/abcEndpoint.cpp") == True;
assert SkipFile("aws-cpp-sdk-core/include/aws/core/Region.h") == True;
assert SkipFile("aws-cpp-sdk-s3/source/model/BucketLocationConstraint.cpp") == True;
assert SkipFile("source/model/abc.cpp") == False;
assert SkipFile("source/model/absEndpoint.cpp") == False;
assert SkipFile("source/model/Endpointabs.cpp") == False;
assert SkipFile("Endpoint.cpp") == False;

assert ScanContent("us-west-1") != None;
assert ScanContent("avbcap-southeast-1") != None;
assert ScanContent("eu-central-1") != None;
assert ScanContent("\"cn-north-1 is in BJS\"") != None;
assert ScanContent("\"cn-north-2 doesn't exist\"") == None;

###################Test End######################################
print "Start checking hard coded endpoints in source files...";
exitCode = 0;
RootDir = os.path.dirname(os.path.dirname(os.path.realpath(__file__)));
for root, dirnames, fileNames in os.walk(RootDir):
    for fileName in fileNames:
        if fileName.endswith(('.h', '.cpp')):
            targetFile = os.path.join(root, fileName);
            exitCode |= CheckFile(targetFile);
print "Finished checking hard coded endpoints in source files with exit code",exitCode,".";
exit(exitCode);
