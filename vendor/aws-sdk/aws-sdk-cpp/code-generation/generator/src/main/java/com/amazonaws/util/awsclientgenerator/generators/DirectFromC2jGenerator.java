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

package com.amazonaws.util.awsclientgenerator.generators;

import com.amazonaws.util.awsclientgenerator.domainmodels.c2j.C2jServiceModel;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import java.io.File;
import java.nio.charset.StandardCharsets;


public class DirectFromC2jGenerator {

    private final MainClientGenerator mainClientGenerator;

    public DirectFromC2jGenerator(final MainClientGenerator mainClientGenerator) {
       this.mainClientGenerator = mainClientGenerator;
    }

    public File generateSourceFromJson(String rawJson, String languageBinding, String serviceName, String namespace, String licenseText, boolean generateStandalonePackage) throws Exception {
        GsonBuilder gsonBuilder = new GsonBuilder();
        Gson gson = gsonBuilder.create();

        C2jServiceModel c2jServiceModel = gson.fromJson(rawJson, C2jServiceModel.class);
        c2jServiceModel.setServiceName(serviceName);
        return mainClientGenerator.generateSourceFromC2jModel(c2jServiceModel, serviceName, languageBinding, namespace, licenseText, generateStandalonePackage);
    }
}
