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

import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ShapeMember;
import org.junit.Test;

import java.util.HashMap;
import java.util.LinkedList;
import java.util.Set;

import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.assertEquals;

public class CppViewHelperTest {

    @Test
    public void testComputeExportValue() {
        assertEquals("AWS_SERVICE_API", CppViewHelper.computeExportValue("service"));
    }

    @Test
    public void testComputeBaseClass() {
        Shape shape = new Shape();
        shape.setPayload("blah");
        shape.setMembers(new HashMap<>());
        ShapeMember payloadShapeMember = new ShapeMember();
        Shape payloadShape = new Shape();
        payloadShape.setType("BlobStream");
        payloadShapeMember.setShape(payloadShape);
        shape.getMembers().put("blah", payloadShapeMember);
        assertEquals("StreamingServiceRequest", CppViewHelper.computeBaseClass("Service", shape));
        shape.setPayload(null);
        shape.setMembers(new HashMap<>());
        ShapeMember shapeMember = new ShapeMember();
        shapeMember.setStreaming(true);
        shape.getMembers().put("ShapeMember", shapeMember);
        assertEquals("StreamingServiceRequest", CppViewHelper.computeBaseClass("Service", shape));
        shape.setMembers(new HashMap<>());
        assertEquals("ServiceRequest", CppViewHelper.computeBaseClass("Service", shape));
    }

    @Test
    public void testComputeMemberVariableName() {
        assertEquals("m_memberVariable", CppViewHelper.computeMemberVariableName("MemberVariable"));
    }

    @Test
    public void testComputeDefaultValue() {
        Shape shape = new Shape();
        shape.setType("long");
        assertEquals("0", CppViewHelper.computeDefaultValue(shape));
        shape.setType("integer");
        assertEquals("0", CppViewHelper.computeDefaultValue(shape));
        shape.setType("float");
        assertEquals("0.0", CppViewHelper.computeDefaultValue(shape));
        shape.setType("boolean");
        assertEquals("false", CppViewHelper.computeDefaultValue(shape));
        shape.setType("double");
        assertEquals("0.0", CppViewHelper.computeDefaultValue(shape));
        shape.setType("not implemented type");
        assertNull(CppViewHelper.computeDefaultValue(shape));
    }

    @Test
    public void testConvertToUpperCamel() {
        assertEquals("UpperCaseVar", CppViewHelper.convertToUpperCamel("upperCaseVar"));
        assertEquals("UpperCaseVar", CppViewHelper.convertToUpperCamel("UpperCaseVar"));
    }

    @Test
    public void testComputeVariableName() {
       assertEquals("memberVariable", CppViewHelper.computeVariableName("MemberVariable"));
    }

    @Test
    public void testVariableHasBeenSetName() {
       assertEquals("m_memberVariableHasBeenSet", CppViewHelper.computeVariableHasBeenSetName("MemberVariable"));
    }

    @Test
    public void testComputeJsonizeString() {
        Shape shape = new Shape();
        shape.setType("structure");
        assertEquals(".Jsonize()", CppViewHelper.computeJsonizeString(shape));
        shape.setType("any thing else");
        assertEquals("", CppViewHelper.computeJsonizeString(shape));
    }

    @Test
    public void testComputeCppType() {
        Shape shape = new Shape();
        assertEquals(false, shape.isSensitive());
        shape.setType("string");
        assertEquals("Aws::String", CppViewHelper.computeCppType(shape));
        shape.setType("long");
        assertEquals("long long", CppViewHelper.computeCppType(shape));
        shape.setType("integer");
        assertEquals("int", CppViewHelper.computeCppType(shape));
        shape.setType("double");
        assertEquals("double", CppViewHelper.computeCppType(shape));
        shape.setType("float");
        assertEquals("double", CppViewHelper.computeCppType(shape));
        shape.setType("boolean");
        assertEquals("bool", CppViewHelper.computeCppType(shape));
        shape.setType("timestamp");
        assertEquals("Aws::Utils::DateTime", CppViewHelper.computeCppType(shape));
        shape.setType("blob");
        assertEquals("Aws::Utils::ByteBuffer", CppViewHelper.computeCppType(shape));
        shape.setSensitive(true);
        assertEquals("Aws::Utils::CryptoBuffer", CppViewHelper.computeCppType(shape));
        shape.setSensitive(false);

        shape.setName("ShapeName");
        shape.setType("structure");
        assertEquals("ShapeName", CppViewHelper.computeCppType(shape));
        shape.setName("EnumShapeName");
        shape.setType("string");
        shape.setEnumValues(new LinkedList<>());
        shape.getEnumValues().add("EnumValue");
        assertEquals("EnumShapeName", CppViewHelper.computeCppType(shape));
        shape.setEnumValues(null);

        Shape listShape = new Shape();
        listShape.setType("list");
        listShape.setName("ListShape");

        Shape innerShape = new Shape();
        innerShape.setType("string");
        innerShape.setName("StringName");

        ShapeMember innerShapeMember = new ShapeMember();
        innerShapeMember.setShape(innerShape);

        listShape.setListMember(innerShapeMember);
        assertEquals("Aws::Vector<Aws::String>", CppViewHelper.computeCppType(listShape));

        Shape mapShape = new Shape();
        mapShape.setType("map");
        mapShape.setName("MapShape");

        Shape keyShape = new Shape();
        keyShape.setType("string");
        keyShape.setName("StringName");

        ShapeMember keyShapeMember = new ShapeMember();
        keyShapeMember.setShape(keyShape);
        mapShape.setMapKey(keyShapeMember);

        Shape structureValue = new Shape();
        structureValue.setType("structure");
        structureValue.setName("StructureShape");

        ShapeMember valueShapeMember = new ShapeMember();
        valueShapeMember.setShape(structureValue);
        mapShape.setMapValue(valueShapeMember);

        assertEquals("Aws::Map<Aws::String, StructureShape>", CppViewHelper.computeCppType(mapShape));

        shape.setType("Any thing else");
        assertEquals("Aws::String", CppViewHelper.computeCppType(shape));
    }

    @Test
    public void testComputeJsonCppType() {
        Shape shape = new Shape();
        shape.setType("long");
        assertEquals("Int64", CppViewHelper.computeJsonCppType(shape));
        shape.setType("integer");
        assertEquals("Integer", CppViewHelper.computeJsonCppType(shape));
        shape.setType("string");
        assertEquals("String", CppViewHelper.computeJsonCppType(shape));
        shape.setType("boolean");
        assertEquals("Bool", CppViewHelper.computeJsonCppType(shape));
        shape.setType("double");
        assertEquals("Double", CppViewHelper.computeJsonCppType(shape));
        shape.setType("double");
        assertEquals("Double", CppViewHelper.computeJsonCppType(shape));
        shape.setType("float");
        assertEquals("Double", CppViewHelper.computeJsonCppType(shape));
        shape.setType("map");
        assertEquals("Object", CppViewHelper.computeJsonCppType(shape));
        shape.setType("list");
        assertEquals("Array", CppViewHelper.computeJsonCppType(shape));
        shape.setType("structure");
        assertEquals("Object", CppViewHelper.computeJsonCppType(shape));
        shape.setType("blob");
        assertEquals("String", CppViewHelper.computeJsonCppType(shape));
        shape.setType("timestamp");
        assertEquals("Double", CppViewHelper.computeJsonCppType(shape));
        shape.setTimestampFormat("iso8601");
        assertEquals("String", CppViewHelper.computeJsonCppType(shape));
        shape.setTimestampFormat("UnixTimestamp");
        assertEquals("Double", CppViewHelper.computeJsonCppType(shape));
        shape.setTimestampFormat("rfc822");
        assertEquals("String", CppViewHelper.computeJsonCppType(shape));
        shape.setType("Any thing else");
        assertNull(CppViewHelper.computeJsonCppType(shape));
    }

    @Test
    public void testComputeHeaderIncludes() {
        Shape structShape = new Shape();
        structShape.setType("structure");
        structShape.setName("StructureShape");

        Shape anotherStructShape = new Shape();
        anotherStructShape.setType("structure");
        anotherStructShape.setName("AnotherStructureShape");

        ShapeMember anotherStructShapeMember = new ShapeMember();
        anotherStructShapeMember.setShape(anotherStructShape);
        structShape.setMembers(new HashMap<>());
        structShape.getMembers().put("AnotherStructureShape", anotherStructShapeMember);

        Shape enumShape = new Shape();
        enumShape.setName("EnumShape");
        enumShape.setType("string");
        enumShape.setEnumValues(new LinkedList<>());
        enumShape.getEnumValues().add("EnumValue");

        ShapeMember enumShapeMember = new ShapeMember();
        enumShapeMember.setShape(enumShape);
        structShape.getMembers().put("EnumShape", enumShapeMember);

        Shape listShape = new Shape();
        listShape.setType("list");
        listShape.setName("ListShape");

        Shape innerShape = new Shape();
        innerShape.setType("string");
        innerShape.setName("StringName");

        ShapeMember innerShapeMember = new ShapeMember();
        innerShapeMember.setShape(innerShape);

        listShape.setListMember(innerShapeMember);
        ShapeMember listShapeMember = new ShapeMember();
        listShapeMember.setShape(listShape);

        structShape.getMembers().put("ListShape", listShapeMember);

        Shape mapShape = new Shape();
        mapShape.setType("map");
        mapShape.setName("MapShape");

        Shape keyShape = new Shape();
        keyShape.setType("string");
        keyShape.setName("StringName");

        ShapeMember keyShapeMember = new ShapeMember();
        keyShapeMember.setShape(keyShape);
        mapShape.setMapKey(keyShapeMember);

        Shape structureValue = new Shape();
        structureValue.setType("structure");
        structureValue.setName("MapValueStructureShape");

        ShapeMember valueShapeMember = new ShapeMember();
        valueShapeMember.setShape(structureValue);
        mapShape.setMapValue(valueShapeMember);

        ShapeMember mapShapeMember = new ShapeMember();
        mapShapeMember.setShape(mapShape);

        structShape.getMembers().put("MapShape", mapShapeMember);

        String serviceAbbr = "service";
        Set<String> headerIncludes = CppViewHelper.computeHeaderIncludes(serviceAbbr, structShape);

        assertTrue(headerIncludes.contains("<aws/" + serviceAbbr + "/model/AnotherStructureShape.h>"));
        assertTrue(headerIncludes.contains("<aws/" + serviceAbbr + "/model/EnumShape.h>"));
        assertTrue(headerIncludes.contains("<aws/" + serviceAbbr + "/model/MapValueStructureShape.h>"));
        assertTrue(headerIncludes.contains("<aws/core/utils/memory/stl/AWSString.h>"));
        assertTrue(headerIncludes.contains("<aws/core/utils/memory/stl/AWSMap.h>"));
        assertTrue(headerIncludes.contains("<aws/core/utils/memory/stl/AWSVector.h>"));

    }

    @Test
    public void testComputeOperationNameFromShapeName() {
        assertEquals("OperationName", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameRequest"));
        assertEquals("OperationName", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameResult"));
        assertEquals("OperationNameOther", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameOther"));

        assertEquals("OperationNameRequests", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameRequestsRequest"));
        assertEquals("OperationNameRequests", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameRequestsResult"));
        assertEquals("OperationNameResults", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameResultsRequest"));
        assertEquals("OperationNameResults", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameResultsResult"));
        assertEquals("OperationNameResultsRequestResultRequest", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameResultsRequestResultRequestResult"));
        assertEquals("OperationNameResultsRequestResultRequest", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameResultsRequestResultRequestRequest"));

        assertEquals("OperationNameResultsRequestResultRequestDerp", CppViewHelper.computeOperationNameFromInputOutputShape("OperationNameResultsRequestResultRequestDerp"));

    }
}
