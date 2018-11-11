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

package com.amazonaws.util.awsclientgenerator.generators.cpp.sqs;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ShapeMember;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppViewHelper;
import com.amazonaws.util.awsclientgenerator.generators.cpp.QueryCppClientGenerator;
import org.apache.velocity.Template;
import org.apache.velocity.VelocityContext;

import java.nio.charset.StandardCharsets;
import java.util.HashMap;

public class SQSQueryXmlCppClientGenerator extends QueryCppClientGenerator {

    public SQSQueryXmlCppClientGenerator() throws Exception {
        super();
    }

    @Override
    public SdkFileEntry[] generateSourceFiles(ServiceModel serviceModel) throws Exception {
        Shape queueAttributeNameShape = serviceModel.getShapes().get("QueueAttributeName");

        //currently SQS doesn't model some values that can be returned as "members" of the QueueAttributeName enum
        //this is not the right solution. The right solution is to add a separate enum for use for ReceiveMessageRequest
        //but backwards compatibility and all that...
        //anyways, add the missing values here.
        if(queueAttributeNameShape != null) {
            queueAttributeNameShape.getEnumValues().add("SentTimestamp");
            queueAttributeNameShape.getEnumValues().add("ApproximateFirstReceiveTimestamp");
            queueAttributeNameShape.getEnumValues().add("ApproximateReceiveCount");
            queueAttributeNameShape.getEnumValues().add("SenderId");
        }

        return super.generateSourceFiles(serviceModel);
    }

    @Override
    protected SdkFileEntry generateClientSourceFile(final ServiceModel serviceModel) throws Exception {
        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/sqs/SQSServiceClientSource.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("source/%sClient.cpp", serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }
}
