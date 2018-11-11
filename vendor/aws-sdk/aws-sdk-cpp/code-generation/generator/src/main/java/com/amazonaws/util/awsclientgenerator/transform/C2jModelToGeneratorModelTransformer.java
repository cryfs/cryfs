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

package com.amazonaws.util.awsclientgenerator.transform;

import com.amazonaws.util.awsclientgenerator.domainmodels.c2j.*;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Error;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.*;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp.CppViewHelper;
import org.apache.commons.lang.WordUtils;

import java.util.*;
import java.util.stream.Collectors;

public class C2jModelToGeneratorModelTransformer {

    private final C2jServiceModel c2jServiceModel;
    Map<String, Shape> shapes;
    Set<String> removedShapes;
    Set<String> removedOperations;
    Map<String, Operation> operations;
    Set<Error> allErrors;
    boolean standalone;

    public C2jModelToGeneratorModelTransformer(C2jServiceModel c2jServiceModel, boolean standalone) {
        this.c2jServiceModel = c2jServiceModel;
        this.standalone = standalone;
    }

    public ServiceModel convert() {
        ServiceModel serviceModel = new ServiceModel();
        serviceModel.setMetadata(convertMetadata());
        serviceModel.setVersion(c2jServiceModel.getVersion());
        serviceModel.setDocumentation(formatDocumentation(c2jServiceModel.getDocumentation(), 3));

        convertShapes();
        convertOperations();
        removeIgnoredOperations();
        removeUnreferencedShapes();
        postProcessShapes();

        serviceModel.setShapes(shapes);
        serviceModel.setOperations(operations);
        serviceModel.setServiceErrors(filterOutCoreErrors(allErrors));
        return serviceModel;
    }

    String formatDocumentation(String documentation, int indentDepth) {
        if(documentation != null) {
            String tabString = "";
            for(int i = 0; i < indentDepth; ++i) {
                tabString += " ";
            }
            String wrappedString = WordUtils.wrap(documentation, 80, System.lineSeparator() + tabString + "* ", false);
            return wrappedString.replace("/*", "/ *").replace("*/", "* /");
        }
        return null;
    }

    String addDocCrossLinks(final String documentation, final String uid, final String shapeOrOperationName) {
        if(documentation != null && uid != null) {
            String seeAlsoRef = String.format("<p><h3>See Also:</h3>   <a href=\"http://docs.aws.amazon.com/goto/WebAPI/%s/%s\">AWS API Reference</a></p>",
                     uid, shapeOrOperationName);

            return documentation + seeAlsoRef;
        }
        return documentation;
    }

    void removeUnreferencedShapes() {
        Iterator<String> iterator = shapes.keySet().iterator();
        while (iterator.hasNext()) {
            String key = iterator.next();
            if (shapes.get(key).getReferencedBy().isEmpty()) {
                iterator.remove();
            }
        }
    }

    Metadata convertMetadata() {
        C2jMetadata c2jMetadata = c2jServiceModel.getMetadata();

        Metadata metadata = new Metadata();
        metadata.setStandalone(standalone);
        metadata.setApiVersion(c2jMetadata.getApiVersion());
        metadata.setConcatAPIVersion(c2jMetadata.getApiVersion().replace("-", ""));
        metadata.setSigningName(c2jMetadata.getSigningName() != null ? c2jMetadata.getSigningName() : c2jMetadata.getEndpointPrefix());
        metadata.setServiceId(c2jMetadata.getServiceId() != null ? c2jMetadata.getServiceId() : c2jMetadata.getEndpointPrefix());

        metadata.setJsonVersion(c2jMetadata.getJsonVersion());
        if("api-gateway".equalsIgnoreCase(c2jMetadata.getProtocol())) {
            metadata.setEndpointPrefix(c2jMetadata.getEndpointPrefix() + ".execute-api");
            metadata.setProtocol("application-json");
            metadata.setStandalone(true);
        } else {
            metadata.setEndpointPrefix(c2jMetadata.getEndpointPrefix());
            metadata.setProtocol(c2jMetadata.getProtocol());
        }
        metadata.setNamespace(c2jMetadata.getServiceAbbreviation());
        metadata.setServiceFullName(c2jMetadata.getServiceFullName());
        metadata.setSignatureVersion(c2jMetadata.getSignatureVersion());
        metadata.setTargetPrefix(c2jMetadata.getTargetPrefix());
        metadata.setGlobalEndpoint(c2jMetadata.getGlobalEndpoint());
        metadata.setTimestampFormat(c2jMetadata.getTimestampFormat());

        if (metadata.getNamespace() == null || metadata.getNamespace().isEmpty()) {
            metadata.setNamespace(sanitizeServiceAbbreviation(metadata.getServiceFullName()));
        } else {
            metadata.setNamespace(sanitizeServiceAbbreviation(metadata.getNamespace()));
        }

        metadata.setClassNamePrefix(CppViewHelper.convertToUpperCamel(ifNotNullOrEmpty(c2jMetadata.getClientClassNamePrefix(), metadata.getNamespace())));

        c2jServiceModel.setServiceName(ifNotNullOrEmpty(c2jServiceModel.getServiceName(), c2jMetadata.getEndpointPrefix()));
        metadata.setProjectName(ifNotNullOrEmpty(c2jMetadata.getClientProjectName(), c2jServiceModel.getServiceName()));

        if(metadata.getProjectName().contains("."))
        {
            metadata.setProjectName(metadata.getProjectName().replace(".", ""));
        }

        return metadata;
    }

    static String ifNotNullOrEmpty(final String target, final String fallback) {
        if (target != null && !target.isEmpty()){
            return target;
        } else {
            return fallback;
        }
    }

    void postProcessShapes() {
        for(Map.Entry<String, Shape> entry : shapes.entrySet()) {
            Shape shape = entry.getValue();

            /*
            If this shape ends up deriving from AmazonStreamingWebServiceRequest, then we already have implemented accessors for ContentType and the
            header insertion there.  So strip this out of the model (affects S3's PutObjectRequest).
            */
            if (shape.hasStreamMembers() && shape.isRequest()) {
                shape.RemoveMember("contentType");
                shape.RemoveMember("ContentType");
            }
        }
    }

    void convertShapes() {
        shapes = new LinkedHashMap<>(c2jServiceModel.getShapes().size());
        removedShapes = new HashSet<String>();

        // First pass adds basic information
        for (Map.Entry<String, C2jShape> entry : c2jServiceModel.getShapes().entrySet()) {
            Shape shape = convertShapeBasics(entry.getValue(), entry.getKey());
            shapes.put(CppViewHelper.convertToUpperCamel(entry.getKey()), shape);
        }

        // Second Pass adds references to other shapes
        for (Map.Entry<String, C2jShape> entry : c2jServiceModel.getShapes().entrySet()) {
            Shape shape = shapes.get(CppViewHelper.convertToUpperCamel(entry.getKey()));
            convertShapeReferences(entry.getValue(), shape);
        }
    }

    Shape convertShapeBasics(C2jShape c2jShape, String shapeName) {

        Shape shape = new Shape();
        HashSet<String> shapesReferencedBy = new HashSet<String>();
        shape.setReferencedBy(shapesReferencedBy);
        shape.setName(CppViewHelper.convertToUpperCamel(shapeName));
        String crossLinkedShapeDocs = addDocCrossLinks(c2jShape.getDocumentation(), c2jServiceModel.getMetadata().getUid(), shape.getName());
        shape.setDocumentation(formatDocumentation(crossLinkedShapeDocs, 3));

        if (c2jShape.getEnums() != null) {
            shape.setEnumValues(new ArrayList<>(c2jShape.getEnums()));
        } else {
            shape.setEnumValues(Collections.emptyList());
        }

        // All shapes only related to shapes enable "eventstream" or "event" should be removed, there are two cases:
        // 1. The removed shape is the only ancestor of this shape.
        // 2. This shape is the ancestor of the removed shape.
        if (c2jShape.isEventstream() || c2jShape.isEvent()) {
            // shape.setIgnored(true);
            removedShapes.add(shape.getName());
        }

        shape.setMax(c2jShape.getMax());
        shape.setMin(c2jShape.getMin());
        shape.setType(c2jShape.getType());
        shape.setLocationName(c2jShape.getLocationName());
        shape.setPayload(c2jShape.getPayload());
        shape.setFlattened(c2jShape.isFlattened());
        shape.setSensitive(c2jShape.isSensitive());
        if("timestamp".equalsIgnoreCase(shape.getType())) {
            // shape's specific timestampFormat overrides the timestampFormat specified in metadata (if any)
            shape.setTimestampFormat(c2jShape.getTimestampFormat() != null ?
                    c2jShape.getTimestampFormat() :
                    c2jServiceModel.getMetadata().getTimestampFormat());
        }
        return shape;
    }

    void convertShapeReferences(C2jShape c2jShape, Shape shape) {

        if (removedShapes.contains(shape.getName())) {
            return;
        }
        
        Map<String, ShapeMember> shapeMemberMap = new LinkedHashMap<>();

        Set<String> required;
        if (c2jShape.getRequired() != null) {
            required = new LinkedHashSet<>(c2jShape.getRequired());
        } else {
            required = Collections.emptySet();
        }

        if (c2jShape.getMembers() != null) {
            c2jShape.getMembers().entrySet().stream().filter(entry -> !entry.getValue().isDeprecated()).forEach(entry -> {
                ShapeMember shapeMember = convertMember(entry.getValue(), shape, required.contains(entry.getKey()));
                shapeMemberMap.put(entry.getKey(), shapeMember);
            });
        }

        shape.setMembers(shapeMemberMap);

        // Shape is a List
        if (c2jShape.getMember() != null && !c2jShape.getMember().isDeprecated()) {
            shape.setListMember(convertMember(c2jShape.getMember(), shape, false));
        }

        if (c2jShape.getKey() != null && !c2jShape.getKey().isDeprecated()) {
            shape.setMapKey(convertMember(c2jShape.getKey(), shape, false));
        }

        if (c2jShape.getValue() != null && !c2jShape.getValue().isDeprecated()) {
            shape.setMapValue(convertMember(c2jShape.getValue(), shape, false));
        }
    }

    ShapeMember convertMember(C2jShapeMember c2jShapeMember, Shape shape, boolean required) {
        ShapeMember shapeMember = new ShapeMember();
        shapeMember.setRequired(required);
        shapeMember.setDocumentation(formatDocumentation(c2jShapeMember.getDocumentation(), 5));
        shapeMember.setFlattened(c2jShapeMember.isFlattened());
        Shape referencedShape = shapes.get(CppViewHelper.convertToUpperCamel(c2jShapeMember.getShape()));
        referencedShape.getReferencedBy().add(shape.getName());
        referencedShape.setReferenced(true);
        shapeMember.setShape(referencedShape);
        shapeMember.setLocationName(c2jShapeMember.getLocationName());
        shapeMember.setLocation(c2jShapeMember.getLocation());
        shapeMember.setQueryName(c2jShapeMember.getQueryName());
        shapeMember.setStreaming(c2jShapeMember.isStreaming());
        shapeMember.setIdempotencyToken(c2jShapeMember.isIdempotencyToken());
        if(shapeMember.isStreaming()) {
            shapeMember.setRequired(true);
        }

        if(shapeMember.isUsedForHeader()) {
           shapeMember.setLocationName(shapeMember.getLocationName().toLowerCase());
        }

        if(c2jShapeMember.getXmlNamespace() != null) {
            shapeMember.setXmlnsUri(c2jShapeMember.getXmlNamespace().getUri());
        }

        return shapeMember;
    }

    void removeIgnoredOperations() {
        // Backward propagation to mark all operations containing removed shapes.
        for (String shapeName : removedShapes) {            
            markRemovedOperations(shapeName);
        }

        // Forward propagation to dereference all shapes related to the operations should be ignored.
        for (String operationName : removedOperations) {
            operations.get(operationName).getRequest().getShape().getReferencedBy().clear();
            dereferenceShape(operations.get(operationName).getRequest().getShape());
            operations.get(operationName).getResult().getShape().getReferencedBy().clear();
            dereferenceShape(operations.get(operationName).getResult().getShape());
            operations.remove(operationName);
        }
    }

    void markRemovedOperations(String name) {
        if (operations.containsKey(name)) {
            removedOperations.add(name);
        }
        else if (shapes.containsKey(name)) {
            Shape shapeShouldIgnore = shapes.get(name);
            for (String shapeName : shapeShouldIgnore.getReferencedBy()) {
                markRemovedOperations(shapeName);
            }
            shapeShouldIgnore.getReferencedBy().clear();
        }
    }

    void dereferenceShape(Shape topShape) {
        if (topShape.getMembers() == null) {
            return;
        }
        for (Map.Entry<String, ShapeMember> entry : topShape.getMembers().entrySet()) {
            entry.getValue().getShape().getReferencedBy().remove(topShape.getName());
            if (entry.getValue().getShape().getReferencedBy().isEmpty()) {
                dereferenceShape(entry.getValue().getShape());
            }
        }
    }

    void convertOperations() {
        allErrors = new HashSet<>();
        operations = new LinkedHashMap<>(c2jServiceModel.getOperations().size());
        removedOperations = new HashSet<>();
        for (Map.Entry<String, C2jOperation> entry : c2jServiceModel.getOperations().entrySet()) {
            if(!entry.getValue().isDeprecated()) {
                operations.put(entry.getKey(), convertOperation(entry.getValue()));
            }
        }
    }

    Operation convertOperation(C2jOperation c2jOperation) {
        Operation operation = new Operation();

        // Documentation
        String crossLinkedShapeDocs =
                addDocCrossLinks(c2jOperation.getDocumentation(), c2jServiceModel.getMetadata().getUid(), c2jOperation.getName());

        operation.setDocumentation(formatDocumentation(crossLinkedShapeDocs, 9));
        operation.setAuthtype(c2jOperation.getAuthtype());
        operation.setAuthorizer(c2jOperation.getAuthorizer());

        // input
        if (c2jOperation.getInput() != null) {
            String requestName = c2jOperation.getName() + "Request";
            Shape requestShape = renameShape(shapes.get(c2jOperation.getInput().getShape()), requestName);
            requestShape.setRequest(true);
            requestShape.setReferenced(true);
            requestShape.getReferencedBy().add(c2jOperation.getName());
            requestShape.setLocationName(c2jOperation.getInput().getLocationName());
            requestShape.setXmlNamespace(c2jOperation.getInput().getXmlNamespace() != null ? c2jOperation.getInput().getXmlNamespace().getUri() : null);

            if(requestShape.getLocationName() != null && requestShape.getLocationName().length() > 0 &&
                    (requestShape.getPayload() == null || requestShape.getPayload().length() == 0) ) {
                requestShape.setPayload(requestName);
            }

            requestShape.setSignBody(true);

            if(operation.getAuthtype() == null) {
                requestShape.setSignerName("Aws::Auth::SIGV4_SIGNER");
            } else if (operation.getAuthtype().equals("v4-unsigned-body")) {
                requestShape.setSignBody(false);
                requestShape.setSignerName("Aws::Auth::SIGV4_SIGNER");
            } else if (operation.getAuthtype().equals("custom")) {
               requestShape.setSignerName("\"" + operation.getAuthorizer() + "\"");
            } else {
                requestShape.setSignerName("Aws::Auth::NULL_SIGNER");
            }

            ShapeMember requestMember = new ShapeMember();
            requestMember.setShape(requestShape);
            requestMember.setDocumentation(formatDocumentation(c2jOperation.getInput().getDocumentation(), 3));

            operation.setRequest(requestMember);
        }

        // output
        if (c2jOperation.getOutput() != null) {
            String resultName = c2jOperation.getName() + "Result";
            Shape resultShape = renameShape(shapes.get(c2jOperation.getOutput().getShape()), resultName);
            resultShape.setResult(true);
            resultShape.setReferenced(true);
            resultShape.getReferencedBy().add(c2jOperation.getName());
            ShapeMember resultMember = new ShapeMember();
            resultMember.setShape(resultShape);
            resultMember.setDocumentation(formatDocumentation(c2jOperation.getOutput().getDocumentation(), 3));
            operation.setResult(resultMember);
        }
        // http
        operation.setHttp(convertHttp(c2jOperation.getHttp()));

        // name
        operation.setName(c2jOperation.getName());

        // errors

        List<Error> operationErrors = new ArrayList<>();
        if (c2jOperation.getErrors() != null) {
            operationErrors.addAll(c2jOperation.getErrors().stream().map(this::convertError).collect(Collectors.toList()));
        }

        operation.setErrors(operationErrors);

        return operation;
    }

    Shape renameShape(Shape shape, String name) {
        if (shape.getName().equals(name)) {
            return shape;
        }
        if (shapes.containsKey(name)) {
            // Conflict with shape name defined by service team, need to rename it.
            String newName = "";
            switch(name) {
                case "CopyObjectResult":
                    newName = "CopyObjectResultDetails";
                    renameShapeMember(shape, name, newName);
                    break;
                case "BatchUpdateScheduleResult":
                    shapes.remove(name);                    
                    break;
                default:
                    throw new RuntimeException("Unhandled shape name conflict: " + name);
            }
        }

        Shape cloned = cloneShape(shape);
        cloned.setName(name);
        shapes.put(name, cloned);
        return cloned;
    }

    Shape cloneShape(Shape shape) {
        Shape cloned = new Shape();
        cloned.setReferencedBy(shape.getReferencedBy());
        cloned.setDocumentation(shape.getDocumentation());
        cloned.setEnumValues(shape.getEnumValues());
        cloned.setListMember(shape.getListMember());
        cloned.setMapKey(shape.getMapKey());
        cloned.setMapValue(shape.getMapValue());
        cloned.setMax(shape.getMax());
        cloned.setMin(shape.getMin());
        cloned.setMembers(shape.getMembers());
        cloned.setResult(shape.isResult());
        cloned.setRequest(shape.isRequest());
        cloned.setType(shape.getType());
        cloned.setPayload(shape.getPayload());
        cloned.setFlattened(shape.isFlattened());
        return cloned;
    }
    void renameShapeMember(Shape parentShape, String originalName, String newName) {
        shapes.get(originalName).setName(newName);
        shapes.put(newName, shapes.get(originalName));
        shapes.remove(originalName);
        parentShape.getMembers().put(newName, parentShape.getMembers().get(originalName));
        parentShape.RemoveMember(originalName);
        parentShape.setPayload(newName);
    }

    Http convertHttp(C2jHttp c2jHttp) {
        Http http = new Http();
        http.setMethod(c2jHttp.getMethod());
        http.setRequestUri(c2jHttp.getRequestUri());
        http.setResponseCode(c2jHttp.getResponseCode());
        return http;
    }

    Error convertError(C2jError c2jError) {
        if(c2jServiceModel.getShapes().get(c2jError.getShape()) != null) {
            C2jShape shape = c2jServiceModel.getShapes().get(c2jError.getShape());
            c2jError.setError(shape.getError());
            c2jError.setException(shape.isException());
        }

        Error error = new Error();
        error.setDocumentation(formatDocumentation(c2jError.getDocumentation(), 3));
        error.setName(c2jError.getShape());
        error.setText(c2jError.getShape());
        error.setException(c2jError.isException());
        error.setFault(c2jError.isFault());

        //query xml loads this inner structure to do this work.
        if (c2jError.getError() != null && c2jError.getError().getCode() != null) {
            if(c2jError.getError().getHttpStatusCode() >= 500 || !c2jError.getError().isSenderFault()) {
                error.setRetryable(true);
            }

            error.setText(c2jError.getError().getCode());
        }

        allErrors.add(error);
        return error;
    }

    Set<Error> filterOutCoreErrors(Set<Error> errors) {
        return errors.stream().filter(e -> !CoreErrors.VARIANTS.contains(e.getName())).collect(Collectors.toSet());
    }

    String sanitizeServiceAbbreviation(String serviceAbbreviation) {
        return serviceAbbreviation.replace(" ", "").replace("-", "").replace("_", "").replace("Amazon", "").replace("AWS", "").replace("/", "");
    }
}
