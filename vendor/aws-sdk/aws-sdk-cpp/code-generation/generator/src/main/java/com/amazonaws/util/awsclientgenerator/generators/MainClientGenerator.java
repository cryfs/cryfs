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

import com.amazonaws.util.awsclientgenerator.SdkSpec;
import com.amazonaws.util.awsclientgenerator.config.ServiceGeneratorConfig;
import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.c2j.C2jServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.transform.C2jModelToGeneratorModelTransformer;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

public class MainClientGenerator {

    public File generateSourceFromC2jModel(C2jServiceModel c2jModel, String serviceName, String languageBinding, String namespace, String licenseText, boolean generateStandalonePackage) throws Exception {

        SdkSpec spec = new SdkSpec(languageBinding, serviceName, null);
        // Transform to ServiceModel
        ServiceModel serviceModel = new C2jModelToGeneratorModelTransformer(c2jModel, generateStandalonePackage).convert();

        serviceModel.setRuntimeMajorVersion("@RUNTIME_MAJOR_VERSION@");
        serviceModel.setRuntimeMajorVersionUpperBound("@RUNTIME_MAJOR_VERSION_UPPER_BOUND@");
        serviceModel.setRuntimeMinorVersion("@RUNTIME_MINOR_VERSION@");
        serviceModel.setNamespace(namespace);
        serviceModel.setLicenseText(licenseText);

        spec.setVersion(serviceModel.getMetadata().getApiVersion());

        String protocol = serviceModel.getMetadata().getProtocol();
        ClientGenerator clientGenerator = ServiceGeneratorConfig.findGenerator(spec, protocol);

        //use serviceName and version to convert the json over.
        SdkFileEntry[] apiFiles = clientGenerator.generateSourceFiles(serviceModel);
        String sdkOutputName = String.format("aws-%s-sdk-%s", spec.getLanguageBinding(), serviceModel.getMetadata().getProjectName());
        File finalOutputFile = File.createTempFile(sdkOutputName, ".zip");

        //we need to add a BOM to accommodate MSFT compilers.
        //as specified here https://blogs.msdn.microsoft.com/vcblog/2016/02/22/new-options-for-managing-character-sets-in-the-microsoft-cc-compiler/
        byte[] bom = {(byte)0xEF,(byte)0xBB,(byte)0xBF};
        FileOutputStream fileOutputStream = new FileOutputStream(finalOutputFile);
        try (ZipOutputStream zipOutputStream = new ZipOutputStream(fileOutputStream, StandardCharsets.UTF_8)) {

            for (SdkFileEntry apiFile : apiFiles) {
                if (apiFile != null && apiFile.getPathRelativeToRoot() != null) {
                    ZipEntry zipEntry = new ZipEntry(String.format("%s/%s", sdkOutputName, apiFile.getPathRelativeToRoot()));
                    zipOutputStream.putNextEntry(zipEntry);

                    if(apiFile.isNeedsByteOrderMark()) {
                        zipOutputStream.write(bom);
                    }

                    zipOutputStream.write(apiFile.getSdkFile().toString().getBytes(StandardCharsets.UTF_8));
                    zipOutputStream.closeEntry();
                }
            }
        }

        return finalOutputFile;
    }

    /**
     * Loads a json file into a service model object.
     *
     * @param path path to the json file.
     * @return Service Model (model of the json object in the specified file)
     * @throws IOException
     */
    private C2jServiceModel loadServiceModelFromFile(final String path) throws IOException {

        StringBuilder inputJson = new StringBuilder();

        try (Reader reader = new InputStreamReader(getClass().getClassLoader().getResourceAsStream(path), StandardCharsets.UTF_8.name())) {
            char[] inputBuffer = new char[1024];

            while (reader.read(inputBuffer) >= 0) {
                inputJson.append(new String(inputBuffer));
            }

            GsonBuilder gsonBuilder = new GsonBuilder();
            Gson gson = gsonBuilder.create();
            return gson.fromJson(inputJson.toString(), C2jServiceModel.class);
        }
    }
}
