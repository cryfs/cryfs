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

package com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp;

import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Metadata;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ShapeMember;
import com.google.common.base.CaseFormat;

import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.LinkedList;
import java.util.Map;
import java.util.Queue;
import java.util.Set;
import java.util.stream.Collectors;

public class CppViewHelper {
    private static final Map<String, String> CORAL_TO_CPP_TYPE_MAPPING = new HashMap<>();
    private static final Map<String, String> CORAL_TO_JSON_CPP_TYPE_MAPPING = new HashMap<>();
    private static final Map<String, String> CORAL_TO_XML_CONVERSION_MAPPING = new HashMap<>();
    private static final Map<String, String> CORAL_TYPE_TO_DEFAULT_VALUES = new HashMap<>();
    private static final Map<String, String> CORAL_TO_CONTENT_TYPE_MAPPING = new HashMap<>();

    static {
        CORAL_TO_CPP_TYPE_MAPPING.put("long", "long long");
        CORAL_TO_CPP_TYPE_MAPPING.put("integer", "int");
        CORAL_TO_CPP_TYPE_MAPPING.put("string", "Aws::String");
        CORAL_TO_CPP_TYPE_MAPPING.put("timestamp", "Aws::Utils::DateTime");
        CORAL_TO_CPP_TYPE_MAPPING.put("boolean", "bool");
        CORAL_TO_CPP_TYPE_MAPPING.put("double", "double");
        CORAL_TO_CPP_TYPE_MAPPING.put("float", "double");
        CORAL_TO_CPP_TYPE_MAPPING.put("blob", "Aws::Utils::ByteBuffer");
        CORAL_TO_CPP_TYPE_MAPPING.put("sensitive_blob", "Aws::Utils::CryptoBuffer");

        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("long", "Int64");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("integer", "Integer");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("string", "String");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("boolean", "Bool");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("double", "Double");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("map", "Object");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("list", "Array");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("structure", "Object");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("blob", "String");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("float", "Double");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("timestamp", "Double");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("unixtimestamp", "Double");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("rfc822", "String");
        CORAL_TO_JSON_CPP_TYPE_MAPPING.put("iso8601", "String");

        CORAL_TO_XML_CONVERSION_MAPPING.put("long", "StringUtils::ConvertToInt64");
        CORAL_TO_XML_CONVERSION_MAPPING.put("integer", "StringUtils::ConvertToInt32");
        CORAL_TO_XML_CONVERSION_MAPPING.put("boolean", "StringUtils::ConvertToBool");
        CORAL_TO_XML_CONVERSION_MAPPING.put("double", "StringUtils::ConvertToDouble");
        CORAL_TO_XML_CONVERSION_MAPPING.put("float", "StringUtils::ConvertToDouble");


        CORAL_TYPE_TO_DEFAULT_VALUES.put("long", "0");
        CORAL_TYPE_TO_DEFAULT_VALUES.put("integer", "0");
        CORAL_TYPE_TO_DEFAULT_VALUES.put("boolean", "false");
        CORAL_TYPE_TO_DEFAULT_VALUES.put("double", "0.0");
        CORAL_TYPE_TO_DEFAULT_VALUES.put("float", "0.0");

        CORAL_TO_CONTENT_TYPE_MAPPING.put("json", "Aws::AMZN_JSON_CONTENT_TYPE_1_1");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("json1.0", "Aws::AMZN_JSON_CONTENT_TYPE_1_0");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("json1.1", "Aws::AMZN_JSON_CONTENT_TYPE_1_1");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("rest-json", "Aws::AMZN_JSON_CONTENT_TYPE_1_1");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("rest-json1.0", "Aws::AMZN_JSON_CONTENT_TYPE_1_0");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("rest-json1.1", "Aws::AMZN_JSON_CONTENT_TYPE_1_1");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("rest-xml", "Aws::AMZN_XML_CONTENT_TYPE");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("query", "Aws::FORM_CONTENT_TYPE");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("ec2", "Aws::FORM_CONTENT_TYPE");
        CORAL_TO_CONTENT_TYPE_MAPPING.put("application-json", "Aws::JSON_CONTENT_TYPE");
    }

    public static String computeExportValue(String classNamePrefix) {
        return String.format("AWS_%s_API", classNamePrefix.toUpperCase());
    }

    public static String computeBaseClass(String classNamePrefix, Shape shape) {
        String streamingName = shape.hasStreamMembers() ? "Streaming" : "";
        return String.format("%s%sRequest", streamingName, classNamePrefix);
    }

    public static String computeMemberVariableName(String memberName) {
        String varName = memberName.substring(0, 1).toLowerCase() + memberName.substring(1);
        return String.format("m_%s", varName);
    }

    public static String computeDefaultValue(Shape shape) {
        return CORAL_TYPE_TO_DEFAULT_VALUES.get(shape.getType());
    }

    public static String computeVariableName(String memberName) {
        return memberName.substring(0, 1).toLowerCase() + memberName.substring(1);
    }

    public static String convertToUpperCamel(String lowerCamel) {
        return CaseFormat.LOWER_CAMEL.to(CaseFormat.UPPER_CAMEL, lowerCamel);
    }

    public static String computeVariableHasBeenSetName(String memberName) {
        return String.format("%sHasBeenSet", computeMemberVariableName(memberName));
    }

    public static String computeJsonizeString(Shape shape) {
        String jsonizeString = ".Jsonize()";

        if(shape.isStructure()) {
            return jsonizeString;
        }

        if(shape.isTimeStamp()) {
            if(shape.getTimestampFormat() == null || CORAL_TO_JSON_CPP_TYPE_MAPPING.get(shape.getTimestampFormat().toLowerCase()).equalsIgnoreCase("Double")) {
                return ".SecondsWithMSPrecision()";
            }

            if(shape.getTimestampFormat().toLowerCase().equalsIgnoreCase("rfc822")) {
                return ".ToGmtString(DateFormat::RFC822)";
            }

            if(shape.getTimestampFormat().toLowerCase().equalsIgnoreCase("iso8601")) {
                return ".ToGmtString(DateFormat::ISO_8601)";
            }
        }

        return "";
    }

    public static String computeCppType(Shape shape) {
        String sensitivePrefix = shape.isSensitive() ? "sensitive_" : "";
        String cppType =  CORAL_TO_CPP_TYPE_MAPPING.get(sensitivePrefix + shape.getType());

        //enum types show up as string
        if(cppType != null && !shape.isEnum()) {
            return cppType;
        }

        else if(shape.isStructure() || shape.isEnum())
        {
            return shape.getName();
        }

        else if(shape.isList()) {
            String type = computeCppType(shape.getListMember().getShape());
            return String.format("Aws::Vector<%s>", type);
        }

        else if(shape.isMap()) {
            String key = computeCppType(shape.getMapKey().getShape());
            String value = computeCppType(shape.getMapValue().getShape());
            return String.format("Aws::Map<%s, %s>", key, value);
        }

        else {
            return "Aws::String";
        }
    }

    public static String computeJsonCppType(Shape shape) {
        if(shape.isTimeStamp() && shape.getTimestampFormat() != null) {
            return CORAL_TO_JSON_CPP_TYPE_MAPPING.get(shape.getTimestampFormat().toLowerCase());
        }
        return CORAL_TO_JSON_CPP_TYPE_MAPPING.get(shape.getType());
    }

    public static String computeXmlConversionMethodName(Shape shape) {
        return CORAL_TO_XML_CONVERSION_MAPPING.get(shape.getType());
    }

    public static String computeRequestContentType(Metadata metadata) {
        String protocolAndVersion = metadata.getProtocol();

        if(metadata.getJsonVersion() != null) {
            protocolAndVersion += metadata.getJsonVersion();
        }

        return CORAL_TO_CONTENT_TYPE_MAPPING.get(protocolAndVersion);
    }

    public static Set<String> computeHeaderIncludes(String projectName, Shape shape) {
        Set<String> headers = new LinkedHashSet<>();
        Set<String> visited = new LinkedHashSet<>();
        Queue<Shape> toVisit = shape.getMembers().values().stream().map(ShapeMember::getShape).collect(Collectors.toCollection(() -> new LinkedList<>()));
        boolean includeUtilityHeader = false;

        while(!toVisit.isEmpty()) {
            Shape next = toVisit.remove();
            visited.add(next.getName());
            if(next.isMap()) {
                if(!visited.contains(next.getMapKey().getShape().getName())) {
                    toVisit.add(next.getMapKey().getShape());
                }
                if(!visited.contains(next.getMapValue().getShape().getName())) {
                    toVisit.add(next.getMapValue().getShape());
                }
            }
            if(next.isList())
            {
                if(!visited.contains(next.getListMember().getShape().getName()))
                {
                    toVisit.add(next.getListMember().getShape());
                }
            }
            if(!next.isPrimitive()) {
                headers.add(formatModelIncludeName(projectName, next));
                includeUtilityHeader = true;
            }
        }

        if(includeUtilityHeader) {
            headers.add("<utility>");
        }

        headers.addAll(shape.getMembers().values().stream().filter(member -> member.isIdempotencyToken()).map(member -> "<aws/core/utils/UUID.h>").collect(Collectors.toList()));
        return headers;
    }

    public static String formatModelIncludeName(String projectName, Shape shape) {

        if(shape.isMap()) {
            return "<aws/core/utils/memory/stl/AWSMap.h>";
        }
        else if(shape.isList()) {
            return "<aws/core/utils/memory/stl/AWSVector.h>";
        }
        else if(shape.isEnum() || shape.isStructure()) {
            return String.format("<aws/%s/model/%s.h>", projectName, shape.getName());
        }
        else if(shape.isString()) {
            return "<aws/core/utils/memory/stl/AWSString.h>";
        }
        else if(shape.isTimeStamp()) {
            return "<aws/core/utils/DateTime.h>";
        }
        else if(shape.isBlob()) {
            return "<aws/core/utils/Array.h>";
        }
        else {
            throw new RuntimeException("Unexpected shape:" + shape.toString());
        }
    }

    public static Set<String> computeSourceIncludes(Shape shape) {
        Set<String> headers = new LinkedHashSet<>();

        for(Map.Entry<String, ShapeMember> entry : shape.getMembers().entrySet()) {
            Shape innerShape = entry.getValue().getShape();
            // if the shape is a blob, list of blobs or a map with a value blob. It's very unlikely that a blob would be
            // the key in a map, but we check it anyways.
            if (innerShape.isBlob() ||
                (innerShape.isList() && innerShape.getListMember().getShape().isBlob()) ||
                (innerShape.isMap() && innerShape.getMapValue().getShape().isBlob()) ||
                (innerShape.isMap() && innerShape.getMapKey().getShape().isBlob())) {
                headers.add("<aws/core/utils/HashingUtils.h>");
            }
            else if(entry.getValue().isUsedForHeader() || entry.getValue().isUsedForQueryString()) {
                headers.add("<aws/core/utils/memory/stl/AWSStringStream.h>");
            }
        }
        return headers;
    }

    public static String computeOperationNameFromInputOutputShape(String shapeName) {
        String requestString = "Request";
        String resultString = "Result";
        int length = shapeName.length();
        int suffixIndex = length;

        if(shapeName.endsWith(requestString)) {
            suffixIndex = length - requestString.length();
        } else if (shapeName.endsWith(resultString)) {
            suffixIndex = length - resultString.length();
        }

        return shapeName.substring(0, suffixIndex);
    }

    public static String capitalizeFirstChar(final String str) {
        if (str.length() > 1) {
            return str.substring(0,1).toUpperCase() + str.substring(1);
        }
        else {
            return str.toUpperCase();
        }
    }
}
