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
import os
import platform
import shutil
import subprocess
import time

def GetPlatformEnvironments():
    return { 'Windows' : { 'destination_directory' : 'C:\\temp' },
             'Linux' : { 'destination_directory' : '/tmp' },
             'Darwin' : { 'destination_directory' : '/tmp' } }

def GetGeneratedSDKs():
    return [    #"acm",
                #"apigateway",
                "autoscaling",
                #"cloudhsm",
                "cloudformation",
                "cloudfront",
                #"cloudsearch",
                #"cloudsearchdomain",
                #"cloudtrail",
                #"codecommit",
                "codedeploy",
                #"codepipeline",
                "cognito-identity",
                #"cognito-sync",
                #"config",
                #"datapipeline",
                #"devicefarm",
                #"directconnect",
                #"ds",
                "dynamodb",
                #"ec2",
                "ecs",
                #"ecr",
                "elasticache",
                "elasticbeanstalk",
                "elasticfilesystem",
                "elasticloadbalancing",
                "elasticmapreduce",
                "elastictranscoder",
                "email",
                #"es",
                #"events",
                #"firehose",
                "gamelift",
                "glacier",
                "iam",
                #"importexport",
                #"inspector",
                #"iot",
                "kinesis",
                "kms",
                "lambda",
                "logs",
                "machinelearning",
                #"marketplacecommerceanalytics",
                "mobileanalytics",
                "monitoring",
                "opsworks",
                "rds",
                "redshift",
                #"route53",
                #"route53domains",
                "s3",
                "sdb",
                "sns",
                "sqs",
                #"ssm",
                #"storagegateway",
                "sts",
                #"support",
                "swf"
                #"waf",
                #"workspaces"
                ]

def GetGeneratedSDKDirectories():
    return [ "aws-cpp-sdk-" + dir for dir in GetGeneratedSDKs() ]

def GetC2JFiles():
    sdks = GetGeneratedSDKs()
    apiDir = os.path.join( os.getcwd(), "code-generation", "api-descriptions" )
    fileList = []
    for baseDir, dirNames, fileNames in os.walk( apiDir ):
        if baseDir == apiDir:
            for fileName in fileNames:
                for sdk in sdks:
                    if fileName.startswith(sdk):
                        fileList.append(fileName)
                        break

    return fileList



def GetTestDirectories():
    return [ "testing-resources", 
             "aws-cpp-sdk-transfer-tests", 
             "aws-cpp-sdk-sqs-integration-tests", 
             "aws-cpp-sdk-s3-integration-tests", 
             "aws-cpp-sdk-redshift-integration-tests",
             "aws-cpp-sdk-lambda-integration-tests",
             "aws-cpp-sdk-identity-management-tests",
             "aws-cpp-sdk-dynamodb-integration-tests",
             "aws-cpp-sdk-cognitoidentity-integration-tests",
             "aws-cpp-sdk-cloudfront-integration-tests",
             "aws-cpp-sdk-core-tests",
             "android-unified-tests" ]

def GetCoreDirectories():
    return [ "aws-cpp-sdk-core", os.path.join("code-generation", "generator", "src"), "scripts", "doxygen", "android-build" ]

def GetHighLevelSDKDirectories():
    return [ "aws-cpp-sdk-access-management",
             "aws-cpp-sdk-identity-management",
             "aws-cpp-sdk-queues",
             "aws-cpp-sdk-transfer" ]

def GetAllDirectories():
    return GetCoreDirectories() + GetGeneratedSDKDirectories() + GetTestDirectories() + GetHighLevelSDKDirectories()

def GetLooseFiles():
    return [ "CMakeLists.txt",
             "LICENSE.txt",
             "NOTICE.txt",
             "README.md",
             os.path.join("toolchains", "android.toolchain.cmake"),
             os.path.join("code-generation", "generator", "LICENSE.txt"),
             os.path.join("code-generation", "generator", "NOTICE.txt"),
             os.path.join("code-generation", "generator", "pom.xml")]


def ParseArguments(platformEnv):
    argMap = {}

    parser = argparse.ArgumentParser(description="AWSNativeSDK 3rdParty Gather Script")
    parser.add_argument("--destdir", action="store")

    args = vars( parser.parse_args() )
    argMap[ "destDir" ] = args[ "destdir" ] or platformEnv['destination_directory']
    
    return argMap



def Main():
    platformEnvironments = GetPlatformEnvironments()
    platformEnv = platformEnvironments[ platform.system() ]
    arguments = ParseArguments(platformEnv)
    
    baseDir = arguments[ "destDir" ]
    sdkDir = "aws-sdk-cpp"
    uploadFilename = "latestSnapshot.zip"
    destDir = os.path.join(baseDir, sdkDir)
    uploadZipFile = os.path.join( baseDir, uploadFilename )

    if os.path.exists( destDir ):
        shutil.rmtree( destDir )

    if os.path.exists( uploadZipFile ):
        os.remove( uploadZipFile )

    time.sleep(2)

    os.makedirs( destDir )

    # copy all files needed
    sourceDir = os.getcwd()
    for dir in GetAllDirectories():
        sourceTree = os.path.join( sourceDir, dir )
        destTree = os.path.join( destDir, dir )
        shutil.copytree( sourceTree, destTree )

    for filename in GetLooseFiles():
        sourceFile = os.path.join( sourceDir, filename )
        destFile = os.path.join(destDir, filename)
        fileDestDir = os.path.dirname(destFile)
        if( not os.path.exists( fileDestDir ) ):
            os.makedirs( fileDestDir )

        shutil.copy( sourceFile, os.path.join(destDir, filename) )

    # c2j files need to be individually filtered to keep out services we don't want
    c2jDir = os.path.join( destDir, "code-generation", "api-descriptions" )
    os.mkdir( c2jDir )
    for c2jFile in GetC2JFiles():
        shutil.copy( os.path.join( sourceDir, "code-generation", "api-descriptions", c2jFile ), c2jDir )

    # zip up the target directory
    os.chdir(baseDir)
    zipCommand = "jar -cMf \"" + uploadZipFile + "\" " + sdkDir
    subprocess.check_call( zipCommand, shell = True )
    os.chdir(sourceDir)

    # shutil.rmtree( destDir )

    print( "Aws SDK for C++  finished 3rd party pre-build gather into: " + uploadZipFile )
    

Main()


