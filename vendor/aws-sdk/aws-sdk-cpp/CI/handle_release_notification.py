
from __future__ import print_function

import json
import zipfile
import boto3
from botocore.exceptions import ClientError

print('Loading function')
bucket_name = 'aws-sdk-cpp-pipeline-sdks-team'
key = 'pending-releases.zip'
temp_archive_file = '/tmp/pending_releases.zip'
artifact = 'pending_releases'
temp_artifact_file = '/tmp/pending_releases'
s3 = boto3.client('s3')

def lambda_handler(event, context):
    message = event['Records'][0]['Sns']['Message']
    print("From SNS: " + message)

    releasesDoc = {}
    releasesDoc['releases'] = []

    pendingReleases = None

    try:
        pendingReleases = s3.get_object(Bucket=bucket_name, Key=key)
        body_stream_to_file(pendingReleases["Body"].read())
        releasesDoc = read_zipped_release_doc()
    except ClientError as e:
        print("Couldn't pull doc, assuming it is empty. exception " + e.message)

    releasesDoc['releases'].append(json.loads(message)["release"])
    write_zipped_release_doc(releasesDoc)
    with open(temp_archive_file) as archive:
        s3.put_object(Bucket=bucket_name, Key=key, Body=archive.read())

    return message
    
def read_zipped_release_doc():
    archive = zipfile.ZipFile(temp_archive_file, 'r')
    with archive.open(artifact) as artifactFile:
        return json.loads(artifactFile.read())
    
def write_zipped_release_doc(doc):
    releasesDocStr = json.dumps(doc)
    print("New Release Doc: " + releasesDocStr)
    with open(temp_artifact_file, "w") as artifactFile:
        artifactFile.write(releasesDocStr)
        
    with zipfile.ZipFile(temp_archive_file, 'w') as archiveStream:
        archiveStream.write(temp_artifact_file, artifact)

def body_stream_to_file(body):
    with open(temp_archive_file, 'w') as archiveFile:
        archiveFile.write(body)

