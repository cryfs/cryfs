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

package com.amazonaws.util.awsclientgenerator.generators.cpp.apigateway;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ShapeMember;
import com.amazonaws.util.awsclientgenerator.generators.cpp.JsonCppClientGenerator;
import com.google.common.collect.Sets;

import java.util.Map;
import java.util.Set;

public class APIGatewayRestJsonCppClientGenerator extends JsonCppClientGenerator {

    public APIGatewayRestJsonCppClientGenerator() throws Exception {
        super();
    }

    public SdkFileEntry[] generateSourceFiles(ServiceModel serviceModel) throws Exception {

        serviceModel.getMetadata().setAcceptHeader("application/json");

        Shape invokeMethodRequest = serviceModel.getShapes().get("TestInvokeMethodRequest");
        Map<String, ShapeMember> members = invokeMethodRequest.getMembers();

        //rename body
        ShapeMember bodyMember = members.get("body");
        members.put("requestBody", bodyMember);
        members.remove("body");

        //rename headers
        ShapeMember headersMember = members.get("headers");
        members.put("requestHeaders", headersMember);
        members.remove("headers");

        Shape authorizerRequest = serviceModel.getShapes().get("TestInvokeAuthorizerRequest");
        members = authorizerRequest.getMembers();

        //rename body
        bodyMember = members.get("body");
        members.put("requestBody", bodyMember);
        members.remove("body");

        //rename headers
        headersMember = members.get("headers");
        members.put("requestHeaders", headersMember);
        members.remove("headers");

        return super.generateSourceFiles(serviceModel);
    }
}
