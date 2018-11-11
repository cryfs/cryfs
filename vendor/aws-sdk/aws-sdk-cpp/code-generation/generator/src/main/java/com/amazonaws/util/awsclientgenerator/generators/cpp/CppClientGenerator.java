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
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Error;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Operation;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppShapeInformation;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppViewHelper;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.EnumModel;
import com.amazonaws.util.awsclientgenerator.generators.ClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.exceptions.SourceGenerationFailedException;
import org.apache.velocity.Template;
import org.apache.velocity.VelocityContext;
import org.apache.velocity.app.VelocityEngine;
import org.apache.velocity.runtime.RuntimeConstants;
import org.apache.velocity.runtime.resource.loader.ClasspathResourceLoader;

import java.io.IOException;
import java.io.StringWriter;
import java.nio.charset.StandardCharsets;
import java.util.*;

public abstract class CppClientGenerator implements ClientGenerator {

    protected final VelocityEngine velocityEngine;

    public CppClientGenerator() throws Exception {
        velocityEngine = new VelocityEngine();
        velocityEngine.setProperty(RuntimeConstants.RESOURCE_LOADER, "classpath");
        velocityEngine.setProperty("classpath.resource.loader.class", ClasspathResourceLoader.class.getName());
        velocityEngine.setProperty(RuntimeConstants.RUNTIME_LOG_LOGSYSTEM_CLASS, "org.apache.velocity.runtime.log.NullLogChute");
        velocityEngine.setProperty("template.provide.scope.control", true);
        velocityEngine.init();
    }

    @Override
    public SdkFileEntry[] generateSourceFiles(ServiceModel serviceModel) throws Exception {

        //for c++, the way serialization works, we want to remove all required fields so we can do a value has been set
        //check on all fields.
        serviceModel.getShapes().values().stream().filter(hasMembers -> hasMembers.getMembers() != null).forEach(shape ->
                shape.getMembers().values().stream().filter(shapeMember ->
                        shapeMember.isRequired()).forEach( member -> member.setRequired(false)));

        getOperationsToRemove().stream().forEach(operation ->
        {
          serviceModel.getOperations().remove(operation);
        });
        List<SdkFileEntry> fileList = new ArrayList<>();
        fileList.addAll(generateModelHeaderFiles(serviceModel));
        fileList.addAll(generateModelSourceFiles(serviceModel));
        fileList.add(generateClientHeaderFile(serviceModel));
        fileList.add(generateClientSourceFile(serviceModel));
        fileList.add(generateRegionHeaderFile(serviceModel));
        fileList.add(generateRegionSourceFile(serviceModel));
        fileList.add(generateErrorsHeaderFile(serviceModel));
        fileList.add(generateErrorMarshallerHeaderFile(serviceModel));
        fileList.add(generateErrorSourceFile(serviceModel));
        fileList.add(generateErrorMarshallingSourceFile(serviceModel));
        fileList.add(generateServiceRequestHeader(serviceModel));
        fileList.add(generateExportHeader(serviceModel));
        fileList.add(generateCmakeFile(serviceModel));
 
        // Currently ec2 Nuget package is over 250MB, which is the hard limit set by Nuget (https://github.com/NuGet/NuGetGallery/issues/6144)
        // So we split ec2 Nuget package to three packages, one for entry, one for Win32 one for x64.
        // Win32 and x64 packages can be set to dependencies to the entry package, so as to keep users experience the same.
        // Arrays.asList("ec2", "s3", "glacier") for additional services.
        if (Arrays.asList("ec2").contains(serviceModel.getMetadata().getProjectName()))
        {
            fileList.addAll(generateNugetFileForLargePackage(serviceModel));
        } else {
            fileList.add(generateNugetFile(serviceModel));
        }

        SdkFileEntry[] retArray = new SdkFileEntry[fileList.size()];
        return fileList.toArray(retArray);
    }

    protected final VelocityContext createContext(final ServiceModel serviceModel) {
        VelocityContext context = new VelocityContext();
        context.put("nl", "\n");
        context.put("serviceModel", serviceModel);
        context.put("input.encoding", StandardCharsets.UTF_8.name());
        context.put("output.encoding", StandardCharsets.UTF_8.name());
        return context;
    }

    protected List<SdkFileEntry> generateModelHeaderFiles(final ServiceModel serviceModel) throws Exception {
        List<SdkFileEntry> sdkFileEntries = new ArrayList<>();

        for (Map.Entry<String, Shape> shapeEntry : serviceModel.getShapes().entrySet()) {
            SdkFileEntry sdkFileEntry = generateModelHeaderFile(serviceModel, shapeEntry);
            if (sdkFileEntry == null) continue;
            sdkFileEntries.add(sdkFileEntry);
        }

        return sdkFileEntries;
    }

    protected SdkFileEntry generateModelHeaderFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        Shape shape = shapeEntry.getValue();
        if (!(shape.isRequest() || shape.isEnum())) {
            return null;
        }

        Template template = null;
        VelocityContext context = createContext(serviceModel);

        if (shape.isRequest()) {
            template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/RequestHeader.vm", StandardCharsets.UTF_8.name());
            for (Map.Entry<String, Operation> opEntry : serviceModel.getOperations().entrySet()) {
                String key = opEntry.getKey();
                Operation op = opEntry.getValue();
                if (op.getRequest() != null && op.getRequest().getShape().getName() == shape.getName()) {
                    context.put("operationName", key);
                    break;
                }
            }
        }
        else if (shape.isEnum()) {
            template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/ModelEnumHeader.vm", StandardCharsets.UTF_8.name());
            EnumModel enumModel = new EnumModel(shapeEntry.getKey(), shape.getEnumValues());
            context.put("enumModel", enumModel);
        }

        context.put("shape", shape);
        context.put("typeInfo", new CppShapeInformation(shape, serviceModel));
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/model/%s.h", serviceModel.getMetadata().getProjectName(),
                shapeEntry.getKey());
        return makeFile(template, context, fileName, true);
    }

    protected List<SdkFileEntry> generateModelSourceFiles(final ServiceModel serviceModel) throws Exception {

        List<SdkFileEntry> sdkFileEntries = new ArrayList<>();

        for (Map.Entry<String, Shape> shapeEntry : serviceModel.getShapes().entrySet()) {

            SdkFileEntry sdkFileEntry = generateModelSourceFile(serviceModel, shapeEntry);
            if (sdkFileEntry != null)
            {
                sdkFileEntries.add(sdkFileEntry);
            }
        }

        return sdkFileEntries;
    }

    protected abstract SdkFileEntry generateErrorMarshallerHeaderFile(ServiceModel serviceModel) throws Exception;

    //these probably don't need to be abstract, since xml and json implementations are not considered here.
    protected abstract SdkFileEntry generateClientHeaderFile(final ServiceModel serviceModel) throws Exception;

    protected abstract SdkFileEntry generateClientSourceFile(final ServiceModel serviceModel) throws Exception;

    protected SdkFileEntry generateModelSourceFile(ServiceModel serviceModel, Map.Entry<String, Shape> shapeEntry) throws Exception {
        Shape shape = shapeEntry.getValue();
        Template template;
        VelocityContext context = createContext(serviceModel);

        if (shape.isEnum()) {
            template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/EnumSource.vm", StandardCharsets.UTF_8.name());
            EnumModel enumModel = new EnumModel(shapeEntry.getKey(), shape.getEnumValues());
            context.put("enumModel", enumModel);

            context.put("shape", shape);
            context.put("typeInfo", new CppShapeInformation(shape, serviceModel));
            context.put("CppViewHelper", CppViewHelper.class);

            String fileName = String.format("source/model/%s.cpp", shapeEntry.getKey());
            return makeFile(template, context, fileName, true);
        }

        return null;
    }

    protected SdkFileEntry generateErrorSourceFile(final ServiceModel serviceModel) throws Exception {

        Set<String> retryableErrors = getRetryableErrors();
        for(Error error : serviceModel.getServiceErrors()) {
           if(retryableErrors.contains(error.getName())) {
               error.setRetryable(true);
           }
        }

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/ServiceErrorsSource.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("ErrorFormatter", ErrorFormatter.class);

        String fileName = String.format("source/%sErrors.cpp", serviceModel.getMetadata().getClassNamePrefix());
        return makeFile(template, context, fileName, true);
    }

    protected Set<String> getRetryableErrors() {
        return new HashSet<>();
    }

    protected SdkFileEntry generateErrorMarshallingSourceFile(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/ServiceErrorMarshallerSource.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("ErrorFormatter", ErrorFormatter.class);

        String fileName = String.format("source/%sErrorMarshaller.cpp", serviceModel.getMetadata().getClassNamePrefix());
        return makeFile(template, context, fileName, true);
    }

    protected SdkFileEntry generateErrorsHeaderFile(ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/ErrorsHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        ErrorFormatter errorFormatter = new ErrorFormatter();
        context.put("errorConstNames", errorFormatter.formatErrorConstNames(serviceModel.getServiceErrors()));
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%sErrors.h", serviceModel.getMetadata().getProjectName(),
                serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }

    protected SdkFileEntry generateNugetFile(ServiceModel serviceModel) throws Exception {
        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/packaging/nuget.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("nl", "\n");

        String fileName = String.format("nuget/aws-cpp-sdk-%s.autopkg", serviceModel.getMetadata().getProjectName());

        return makeFile(template, context, fileName, true);
    }

    protected List<SdkFileEntry> generateNugetFileForLargePackage(ServiceModel serviceModel) throws Exception {

        Template entryTemplate = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/packaging/LargePackageEntryNuget.vm", StandardCharsets.UTF_8.name());
        Template win32Template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/packaging/LargePackageWin32Nuget.vm", StandardCharsets.UTF_8.name());
        Template x64Template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/packaging/LargePackageX64Nuget.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("nl", "\n");

        String entryFileName = String.format("nuget/aws-cpp-sdk-%s.autopkg", serviceModel.getMetadata().getProjectName());
        String win32FileName = String.format("nuget/aws-cpp-sdk-%s.win32.autopkg", serviceModel.getMetadata().getProjectName());
        String x64FileName = String.format("nuget/aws-cpp-sdk-%s.x64.autopkg", serviceModel.getMetadata().getProjectName());

        List<SdkFileEntry> fileList = new ArrayList<>();
        fileList.add(makeFile(entryTemplate, context, entryFileName, true));
        fileList.add(makeFile(win32Template, context, win32FileName, true));
        fileList.add(makeFile(x64Template, context, x64FileName, true));

        return fileList;
    }


    private SdkFileEntry generateServiceRequestHeader(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/AbstractServiceRequest.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%sRequest.h", serviceModel.getMetadata().getProjectName(),
                serviceModel.getMetadata().getClassNamePrefix());

        return makeFile(template, context, fileName, true);
    }

    private SdkFileEntry generateRegionHeaderFile(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/EndpointEnumHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("exportValue", String.format("AWS_%s_API", serviceModel.getMetadata().getClassNamePrefix().toUpperCase()));

        String fileName = String.format("include/aws/%s/%s%s.h", serviceModel.getMetadata().getProjectName(),
                serviceModel.getMetadata().getClassNamePrefix(), "Endpoint");

        return makeFile(template, context, fileName, true);
    }

    private SdkFileEntry generateRegionSourceFile(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/EndpointEnumSource.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("endpointMapping", computeRegionEndpointsForService(serviceModel));

        String fileName = String.format("source/%s%s.cpp", serviceModel.getMetadata().getClassNamePrefix(), "Endpoint");
        return makeFile(template, context, fileName, true);
    }

    protected Map<String, String> computeRegionEndpointsForService(final ServiceModel serviceModel) {
        return new LinkedHashMap<>();
    }

    private SdkFileEntry generateExportHeader(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/ServiceExportHeader.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);
        context.put("CppViewHelper", CppViewHelper.class);

        String fileName = String.format("include/aws/%s/%s_EXPORTS.h", serviceModel.getMetadata().getProjectName(),
                serviceModel.getMetadata().getClassNamePrefix());
        return makeFile(template, context, fileName, true);
    }

    private SdkFileEntry generateCmakeFile(final ServiceModel serviceModel) throws Exception {

        Template template = velocityEngine.getTemplate("/com/amazonaws/util/awsclientgenerator/velocity/cpp/CMakeFile.vm", StandardCharsets.UTF_8.name());

        VelocityContext context = createContext(serviceModel);

        return makeFile(template, context, "CMakeLists.txt", false);
    }

    protected final SdkFileEntry makeFile(Template template, VelocityContext context, String path, boolean needsBOM) throws IOException {
        StringWriter sw = new StringWriter();
        template.merge(context, sw);

        try {
            sw.close();
        } catch (IOException e) {
            throw new SourceGenerationFailedException(String.format("Generation of template failed for template %s", template.getName()), e);
        }
        sw.flush();
        StringBuffer sb = new StringBuffer();
        sb.append(sw.toString());

        SdkFileEntry file = new SdkFileEntry();
        file.setPathRelativeToRoot(path);
        file.setSdkFile(sb);
        file.setNeedsByteOrderMark(needsBOM);
        return file;
    }

    protected Set<String> getOperationsToRemove(){
        return new HashSet<String>();
    }
}
