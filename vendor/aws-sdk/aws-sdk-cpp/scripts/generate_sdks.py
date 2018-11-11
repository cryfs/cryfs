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
import shutil
import subprocess
import re
import subprocess
import os
import zipfile
import io
import codecs
from subprocess import PIPE, STDOUT, Popen
from os import listdir
from os.path import isfile, join

def ParseArguments():
    argMap = {}

    parser = argparse.ArgumentParser(description="Generates an SDK given an sdk name and version")
    parser.add_argument("--outputLocation", action="store")
    parser.add_argument("--serviceName", action="store")
    parser.add_argument("--apiVersion", action="store")
    parser.add_argument("--namespace", action="store")
    parser.add_argument("--licenseText", action="store")
    parser.add_argument("--pathToApiDefinitions", action="store")
    parser.add_argument("--pathToGenerator", action="store")
    parser.add_argument("--prepareTools", help="Makes sure generation environment is setup.", action="store_true")
    parser.add_argument("--listAll", help="Lists all available SDKs for generation.", action="store_true")

    args = vars( parser.parse_args() )
    argMap[ "outputLocation" ] = args[ "outputLocation" ] or "./"
    argMap[ "serviceName" ] = args[ "serviceName" ] or None
    argMap[ "apiVersion" ] = args[ "apiVersion" ] or ""
    argMap[ "namespace" ] = args[ "namespace" ] or ""
    argMap[ "licenseText" ] = args[ "licenseText" ] or ""
    argMap[ "pathToApiDefinitions" ] = args["pathToApiDefinitions"] or "./code-generation/api-descriptions"
    argMap[ "pathToGenerator" ] = args["pathToGenerator"] or "./code-generation/generator"
    argMap[ "prepareTools" ] = args["prepareTools"]
    argMap[ "listAll" ] = args["listAll"]

    return argMap

serviceNameRemaps = {
    "runtime.lex" : "lex",
    "entitlement.marketplace" : "marketplace-entitlement",
    "runtime.sagemaker" : "sagemaker-runtime"
}

def DiscoverAllAvailableSDKs(discoveryPath):
    sdks = {}

    filesInDir = [f for f in listdir(discoveryPath) if isfile(join(discoveryPath, f))]

    for file in filesInDir:
        match = re.search('([\w\d\.-]+)-(\d{4}-\d{2}-\d{2}).normal.json', file)
        if match:
            serviceName = match.group(1)
            if serviceName in serviceNameRemaps:
                serviceName = serviceNameRemaps[serviceName]

            sdk = {}
            sdk['serviceName'] = serviceName
            sdk['apiVersion'] = match.group(2)
            sdk['filePath'] = join(discoveryPath, file)
            sdks['{}-{}'.format(sdk['serviceName'], sdk['apiVersion'])] = sdk

    return sdks

def PrepareGenerator(generatorPath):
    currentDir = os.getcwd()
    os.chdir(generatorPath)
    process = subprocess.call('mvn package', shell=True)
    os.chdir(currentDir)

def GenerateSdk(generatorPath, sdk, outputDir, namespace, licenseText):
    try:
       with codecs.open(sdk['filePath'], 'rb', 'utf-8') as api_definition:
            api_content = api_definition.read()
            jar_path = join(generatorPath, 'target/aws-client-generator-1.0-SNAPSHOT-jar-with-dependencies.jar')
            process = Popen(['java', '-jar', jar_path, '--service', sdk['serviceName'], '--version', sdk['apiVersion'], '--namespace', namespace, '--license-text', licenseText, '--language-binding', 'cpp', '--arbitrary'],stdout=PIPE,  stdin=PIPE)
            writer = codecs.getwriter('utf-8')
            stdInWriter = writer(process.stdin)
            stdInWriter.write(api_content)
            process.stdin.close()
            output = process.stdout.read()
            if output:
                 with zipfile.ZipFile(output.strip().decode('utf-8'), 'r') as zip:
                     zip.extractall(outputDir)
    except EnvironmentError as  ex:
        print('Error generating sdk {} with error {}'.format(sdk, ex))

def Main():
    arguments = ParseArguments()

    if arguments['prepareTools']:
        PrepareGenerator(arguments['pathToGenerator'])

    sdks = DiscoverAllAvailableSDKs(arguments['pathToApiDefinitions'])

    if arguments['listAll']:
        for key, value in sdks.iteritems():
            print(value)

    if arguments['serviceName']:
        print('Generating {} api version {}.'.format(arguments['serviceName'], arguments['apiVersion']))
        key = '{}-{}'.format(arguments['serviceName'], arguments['apiVersion'])
        GenerateSdk(arguments['pathToGenerator'], sdks[key], arguments['outputLocation'], arguments['namespace'], arguments['licenseText'])

Main()
