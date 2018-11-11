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

package com.amazonaws.util.awsclientgenerator.generators.cpp.lambda;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Operation;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.generators.cpp.JsonCppClientGenerator;
import com.google.common.collect.Sets;

import java.util.Set;

public class LambdaRestJsonCppClientGenerator extends JsonCppClientGenerator {

    public LambdaRestJsonCppClientGenerator() throws Exception {
        super();
    }

    @Override
    protected Set<String> getOperationsToRemove() {
        //InvokeAsync collides with our Async client generation.
        //It is deprecated, so we're just not going to generate it
        return Sets.newHashSet("InvokeAsync");
    }

    public SdkFileEntry[] generateSourceFiles(ServiceModel serviceModel) throws Exception {

        serviceModel.getShapes().remove("InvokeAsyncRequest");
        serviceModel.getShapes().remove("InvokeAsyncResult");

        return super.generateSourceFiles(serviceModel);
    }
}
