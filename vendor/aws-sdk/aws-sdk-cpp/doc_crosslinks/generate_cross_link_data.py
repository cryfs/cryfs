import os
import argparse
import io
import codecs
import json
import re
from os import listdir
from os.path import isfile, join

def generateDocsMap(apiDefinitionsPath):
    sdks = {}

    filesInDir = [f for f in listdir(apiDefinitionsPath) if isfile(join(apiDefinitionsPath, f))]

    for file in filesInDir:
        match = re.search('([\w\d-]+)-(\d{4}-\d{2}-\d{2}).normal.json', file)
        if match:
            with codecs.open(join(apiDefinitionsPath, file), 'rb', 'utf-8') as api_definition:
                api_content = json.loads(api_definition.read())
                if "uid" in api_content["metadata"].keys():
                    sdks[api_content["metadata"]["uid"]] = getServiceNameFromMetadata(api_content["metadata"])
           
    return sdks
    
def getServiceNameFromMetadata(metadataNode):
    toSanitize = ""
    if "serviceAbbreviation" in metadataNode.keys():
        toSanitize = metadataNode["serviceAbbreviation"]
    else: 
        toSanitize = metadataNode["serviceFullName"]
          
    return toSanitize.replace(" ","").replace("-", "").replace("_", "").replace("Amazon", "").replace("AWS", "").replace("/", "")

def insertDocsMapToRedirect(apiDefinitionsPath, templatePath, outputPath):
    sdks = generateDocsMap(apiDefinitionsPath)
    
    output = ""
    with codecs.open(templatePath, 'rb', 'utf-8') as redirect_template:
        current_template = redirect_template.read();
        output = current_template.replace("${UID_SERVICE_MAPPING}", json.dumps(sdks, ensure_ascii=False))
    with open(outputPath, 'w') as redirect_output:
        redirect_output.write(output)
                
def Main():
    parser = argparse.ArgumentParser(description="Generates a Cross-link redirect file.")
    parser.add_argument("--apiDefinitionsPath", action="store")
    parser.add_argument("--templatePath", action="store")
    parser.add_argument("--outputPath", action="store")
    
    args = vars( parser.parse_args() )
    argMap = {}
    argMap[ "apiDefinitionsPath" ] = args[ "apiDefinitionsPath" ] or "../code-generation/api-descriptions"
    argMap[ "templatePath" ] = args[ "templatePath" ] or "./"
    argMap[ "outputPath" ] = args[ "outputPath" ] or "/"
    
    insertDocsMapToRedirect(argMap["apiDefinitionsPath"], argMap["templatePath"], argMap["outputPath"])
    
Main()    
            