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

package com.amazonaws.util.awsclientgenerator.generators.cpp.cloudfront;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppShapeInformation;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppViewHelper;
import com.amazonaws.util.awsclientgenerator.generators.cpp.RestXmlCppClientGenerator;
import org.apache.velocity.Template;
import org.apache.velocity.VelocityContext;

import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;

public class CloudfrontCppClientGenerator extends RestXmlCppClientGenerator {

    public CloudfrontCppClientGenerator() throws Exception {
        super();
    }

    @Override
    protected Map<String, String> computeRegionEndpointsForService(final ServiceModel serviceModel) {
        Map<String, String> endpoints = new HashMap<>();
        endpoints.put("us-east-1", serviceModel.getMetadata().getGlobalEndpoint());

        return endpoints;
    }

    @Override
    protected SdkFileEntry generateModelSourceFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        Shape shape = shapeEntry.getValue();

        if (shape.isResult()) {
            VelocityContext context = createContext(serviceModel);
            Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/xml/rest/CloudfrontXmlResultSource.vm", StandardCharsets.UTF_8.name());
            context.put("shape", shape);
            context.put("typeInfo", new CppShapeInformation(shape, serviceModel));
            context.put("CppViewHelper", CppViewHelper.class);

            String fileName = String.format("source/model/%s.cpp", shapeEntry.getKey());

            return makeFile(template, context, fileName, true);
        }

        return super.generateModelSourceFile(serviceModel, shapeEntry);
    }
}