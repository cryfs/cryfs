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
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ShapeMember;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppShapeInformation;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppViewHelper;
import org.apache.velocity.Template;
import org.apache.velocity.VelocityContext;

import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;

public class QueryCppClientGenerator extends CppClientGenerator {

    public QueryCppClientGenerator() throws Exception {
        super();
    }

    @Override
    public SdkFileEntry[] generateSourceFiles(ServiceModel serviceModel) throws Exception {
        Shape shape = new Shape();
        shape.setName("ResponseMetadata");
        shape.setReferenced(true);
        shape.setType("structure");

        Shape stringShape = new Shape();
        stringShape.setName("RequestId");
        stringShape.setType("string");

        ShapeMember stringShapeMember = new ShapeMember();
        stringShapeMember.setShape(stringShape);
        shape.setMembers(new HashMap<>());
        shape.getMembers().put("RequestId", stringShapeMember);

        serviceModel.getShapes().put("ResponseMetadata", shape);

        ShapeMember responseMetadataMember = new ShapeMember();
        responseMetadataMember.setShape(shape);
        responseMetadataMember.setRequired(true);

        for(Shape resultShape : serviceModel.getShapes().values()) {
            if(resultShape.isResult()) {
                resultShape.getMembers().put("ResponseMetadata", responseMetadataMember);
            }
        }

        //query api ALWAYS needs a request shape, because it needs to send action and version as part of the payload
        //we don't want to add it to the operation however, because there is no need for the user to be aware of the existence of this
        //type.
        serviceModel.getOperations().values().stream().filter(operation -> operation.getRequest() == null).forEach(operation -> {
            Shape requestShape = new Shape();
            requestShape.setName(operation.getName() + "Request");
            requestShape.setReferenced(true);
            requestShape.setRequest(true);
            requestShape.setType("structure");
            requestShape.setMembers(new HashMap<>());
            requestShape.setSupportsPresigning(true);

            serviceModel.getShapes().put(requestShape.getName(), requestShape);
            ShapeMember shapeMemberForRequest = new ShapeMember();
            shapeMemberForRequest.setDocumentation("");
            shapeMemberForRequest.setShape(requestShape);
            operation.setRequest(shapeMemberForRequest);
        });

        serviceModel.getOperations().values().stream().forEach(operation -> {
            operation.getRequest().getShape().setSupportsPresigning(true);
        });

        return super.generateSourceFiles(serviceModel);
    }

    @Override
    protected SdkFileEntry generateModelHeaderFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        Shape shape = shapeEntry.getValue();

        //we only want to handle results and internal structures. We don't want requests or enums.
        if (shape.isRequest() || shape.isEnum()) {
            return super.generateModelHeaderFile(serviceModel, shapeEntry);
        }

        if (shape.isStructure() && shape.isReferenced()) {
            Template template = null;
            VelocityContext context = createContext(serviceModel);

            if (shape.isResult()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/xml/XmlResultHeader.vm", StandardCharsets.UTF_8.name());
            } else if (!shape.isRequest() && shape.isStructure()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/queryxml/QueryXmlSubObjectHeader.vm", StandardCharsets.UTF_8.name());
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

        Template template = null;
        VelocityContext context = createContext(serviceModel);

        if (shape.isStructure() && shape.isReferenced()) {
            if (shape.isRequest()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/queryxml/QueryRequestSource.vm", StandardCharsets.UTF_8.name());
            } else if (shape.isResult()) {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/queryxml/QueryXmlResultSource.vm", StandardCharsets.UTF_8.name());
            } else {
                template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/queryxml/QueryXmlSubObjectSource.vm", StandardCharsets.UTF_8.name());
            }
        }

        context.put("shape", shape);
        context.put("typeInfo", new CppShapeInformation(shape, serviceModel));
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("source/model/%s.cpp", shapeEntry.getKey());
        if (template == null)
            return null;

        return makeFile(template, context, fileName, true);
    }

    @Override
    protected SdkFileEntry generateClientHeaderFile(final ServiceModel serviceModel) throws Exception {
        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/xml/XmlServiceClientHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%sClient.h", serviceModel.getMetadata().getProjectName(),
                serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }

    @Override
    protected SdkFileEntry generateClientSourceFile(final ServiceModel serviceModel) throws Exception {
        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/queryxml/QueryXmlServiceClientSource.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("source/%sClient.cpp", serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }

    @Override
    protected SdkFileEntry generateErrorMarshallerHeaderFile(ServiceModel serviceModel) throws Exception {
        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/xml/XmlErrorMarshallerHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%sErrorMarshaller.h",
                serviceModel.getMetadata().getProjectName(), serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }
}
