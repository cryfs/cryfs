
from __future__ import print_function

import json
import zipfile
import boto3
import os
import re
import sys
import argparse
from botocore.exceptions import ClientError
import requests

import requests.packages.urllib3
requests.packages.urllib3.disable_warnings()

temp_archive_file = 'models.zip'

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-r', '--releaseDoc')
    parser.add_argument('-m', '--modelsDir')
    args = parser.parse_args()
   
    releaseDocPath = args.releaseDoc
    modelsDir = args.modelsDir 
   
    print('Release Doc path {0}'.format(releaseDocPath))
    print('Models Directory {0}'.format(modelsDir))
 
    releaseDoc = {}

    pendingReleases = None
    
    with open(releaseDocPath, "r") as releaseDocFileStream:
        releaseDoc = json.loads(releaseDocFileStream.read())
        
    if(len(releaseDoc) == 0 or len(releaseDoc["releases"]) == 0):
        return
        
    for release in releaseDoc["releases"]:
        for feature in release["features"]:
            if feature["c2jModels"] != None:
                response = requests.get(feature["c2jModels"])
                if response.status_code != 200:
                    print("Error downloading {0} artifacts skipping.", json.dumps(feature))
                    continue
                
                body_stream_to_file(response.content)
                copy_model_files(modelsDir)
            cat_release_notes(feature["releaseNotes"], modelsDir)
           
        cat_pending_releases(release["id"], modelsDir)

    emptyReleaseDoc = "{ \"releases\": []}"

    with open(releaseDocPath, "w") as emptyReleaseFile:
        emptyReleaseFile.write(emptyReleaseDoc)
            
def copy_model_files(models_dir):
    archive = zipfile.ZipFile(temp_archive_file, 'r')
    archive.debug = 3
    for info in archive.infolist():
        print(info.filename)
        if re.match(r'output/.*\.normal\.json', info.filename):
            outputPath = os.path.join(models_dir, os.path.basename(info.filename))
            print("copying {0} to {1}".format(info.filename, outputPath))
            fileHandle = archive.open(info.filename, 'r')
            fileOutput = fileHandle.read()
            
            with open(outputPath, 'wb') as destination:
               destination.write(fileOutput)
           
            fileHandle.close()
    
def body_stream_to_file(body):
    with open(temp_archive_file, 'w') as archiveFile:
        archiveFile.write(body)
        
def cat_release_notes(releaseNotes, models_path):
    with open(os.path.join(models_path, "release_notes"), "a") as releaseNotesFile:
        releaseNotesFile.write(releaseNotes + "\n\n") 

def cat_pending_releases(release_guid, models_path):
    with open(os.path.join(models_path, "pending_releases"), "a") as pendingReleasesFile:
        pendingReleasesFile.write(release_guid + "\n")        

if __name__ == "__main__":
    main()
