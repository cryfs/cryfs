/*
* Copyright 2010-2017 Amazon.com, Inc. or its affiliates. All Rights Reserved.
*
* Licensed under the Apache License, Version 2.0 (the "License").
* You may not use this file except in compliance with the License.
* A copy of the License is located at
*
*  http://aws.amazon.com/apache2.0
*
* or in the "license" file accompanying this file. This file is distributed
* on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
* express or implied. See the License for the specific language governing
* permissions and limitations under the License.
*/

package com.amazonaws.util.awsclientgenerator.generators.cpp.dynamodb;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.generators.cpp.JsonCppClientGenerator;
import org.apache.velocity.Template;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

public class DynamoDBJsonCppClientGenerator extends JsonCppClientGenerator {

    public DynamoDBJsonCppClientGenerator() throws Exception {
        super();
    }

    @Override
    public SdkFileEntry[] generateSourceFiles(ServiceModel serviceModel) throws Exception {

        // add a helper class that is used for AttributeValue values.
        Shape attributeValueShape = new Shape();
        attributeValueShape.setName("AttributeValueValue");
        attributeValueShape.setType("structure");
        serviceModel.getShapes().put(attributeValueShape.getName(), attributeValueShape);

        return super.generateSourceFiles(serviceModel);
    }

    @Override
    protected SdkFileEntry generateModelHeaderFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        switch(shapeEntry.getKey()) {
            case "AttributeValue": {
                Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/dynamodb/AttributeValueHeader.vm", StandardCharsets.UTF_8.name());
                return makeFile(template, createContext(serviceModel), "include/aws/dynamodb/model/AttributeValue.h", true);
            }
            case "AttributeValueValue": {
                Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/dynamodb/AttributeValueValueHeader.vm", StandardCharsets.UTF_8.name());
                return makeFile(template, createContext(serviceModel), "include/aws/dynamodb/model/AttributeValueValue.h", true);
            }
            default:
                return super.generateModelHeaderFile(serviceModel, shapeEntry);
        }
    }

    @Override
    protected SdkFileEntry generateModelSourceFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        switch(shapeEntry.getKey()) {
            case "AttributeValue": {
                Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/dynamodb/AttributeValueSource.vm");
                return makeFile(template, createContext(serviceModel), "source/model/AttributeValue.cpp", true);
            }
            case "AttributeValueValue": {
                Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/dynamodb/AttributeValueValueSource.vm");
                return makeFile(template, createContext(serviceModel), "source/model/AttributeValueValue.cpp", true);
            }
            default:
                return super.generateModelSourceFile(serviceModel, shapeEntry);
        }
    }

    @Override
    protected Set<String> getRetryableErrors() {
        return new HashSet<>(Arrays.asList(new String[]{"LimitExceededException", "ProvisionedThroughputExceededException", "ResourceInUseException"}));
    }
}
