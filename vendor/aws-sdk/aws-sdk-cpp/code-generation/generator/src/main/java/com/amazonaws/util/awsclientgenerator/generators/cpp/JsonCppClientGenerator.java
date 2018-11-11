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

package com.amazonaws.util.awsclientgenerator.generators.cpp;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppShapeInformation;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppViewHelper;
import org.apache.velocity.Template;
import org.apache.velocity.VelocityContext;

import java.nio.charset.StandardCharsets;
import java.util.Map;

public class JsonCppClientGenerator extends CppClientGenerator {

    public JsonCppClientGenerator() throws Exception {
        super();
    }

    @Override
    protected SdkFileEntry generateErrorMarshallerHeaderFile(ServiceModel serviceModel) throws Exception {
        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonErrorMarshallerHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%sErrorMarshaller.h",
                serviceModel.getMetadata().getProjectName(), serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }

    @Override
    protected SdkFileEntry generateModelHeaderFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {

        Shape shape = shapeEntry.getValue();
        //we only want to override json related stuff.
        if (shape.isRequest() || shape.isEnum()) {
            return super.generateModelHeaderFile(serviceModel, shapeEntry);
        }

        if (shape.isStructure() && shape.isReferenced()) {
            Template template = null;
            VelocityContext context = createContext(serviceModel);

            if (shape.isResult() && shape.hasStreamMembers()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/StreamResultHeader.vm", StandardCharsets.UTF_8.name());
            }
            else if (shape.isResult()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonResultHeader.vm", StandardCharsets.UTF_8.name());
            } else if (!shape.isRequest() && shape.isStructure()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonSubObjectHeader.vm", StandardCharsets.UTF_8.name());
            }

            context.put("shape", shape);
            context.put("typeInfo", new CppShapeInformation(shape, serviceModel));
            context.put("CppViewHelper", CppViewHelper.class);

            String fileName = String.format("include/aws/%s/model/%s.h", serviceModel.getMetadata().getProjectName(),
                    shapeEntry.getKey());
            return makeFile(template, context, fileName, true);
        }
        return null;
    }

    @Override
    protected SdkFileEntry generateModelSourceFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        Shape shape = shapeEntry.getValue();
        if (shape.isEnum()) {
            return super.generateModelSourceFile(serviceModel, shapeEntry);
        }

        if (shape.isStructure() && shape.isReferenced()) {
            Template template;
            VelocityContext context = createContext(serviceModel);

            if (shape.isRequest() && shape.hasStreamMembers()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/StreamRequestSource.vm", StandardCharsets.UTF_8.name());
            }
            else if (shape.isRequest()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonRequestSource.vm", StandardCharsets.UTF_8.name());
            }
            else if (shape.isResult() && shape.hasStreamMembers()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/StreamResultSource.vm", StandardCharsets.UTF_8.name());
            }
            else if (shape.isResult()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonResultSource.vm", StandardCharsets.UTF_8.name());
            } else {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonSubObjectSource.vm", StandardCharsets.UTF_8.name());
            }

            context.put("shape", shape);
            context.put("typeInfo", new CppShapeInformation(shape, serviceModel));
            context.put("CppViewHelper", CppViewHelper.class);
            context.put("presignerTemplate", "/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonDumpBodyToUrl.vm");

            String fileName = String.format("source/model/%s.cpp", shapeEntry.getKey());

            return makeFile(template, context, fileName, true);
        }
        return null;
    }

    @Override
    protected SdkFileEntry generateClientHeaderFile(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonServiceClientHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%sClient.h", serviceModel.getMetadata().getProjectName(),
                serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }

    @Override
    protected SdkFileEntry generateClientSourceFile(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/json/JsonServiceClientSource.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("source/%sClient.cpp", serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }
}
