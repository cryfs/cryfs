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

import org.junit.Test;

import static java.util.Arrays.asList;
import static org.junit.Assert.assertEquals;

public class EnumModelTest {

    @Test
    public void invalidChars() {

        EnumModel testEnum = new EnumModel("TESTENUM",
                asList("PACKAGE.NAME", "HYPHENS-ROCK", "OH:DARK:THIRTY"));

        assertEquals(3, testEnum.getMembers().size());
        assertEquals("PACKAGE_NAME", testEnum.getMembers().get(0).getMemberName());
        assertEquals("HYPHENS_ROCK", testEnum.getMembers().get(1).getMemberName());
        assertEquals("OH_DARK_THIRTY", testEnum.getMembers().get(2).getMemberName());
    }

    @Test
    public void invalidWord() {

        EnumModel testEnum = new EnumModel("TESTENUM", asList("DELETE"));

        assertEquals(1, testEnum.getMembers().size());
        assertEquals("DELETE_", testEnum.getMembers().get(0).getMemberName());
    }
}
