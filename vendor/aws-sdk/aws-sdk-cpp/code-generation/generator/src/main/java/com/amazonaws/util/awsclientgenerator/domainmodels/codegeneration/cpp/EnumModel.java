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

import lombok.Data;

import java.util.*;

@Data
public class EnumModel {
    private String name;
    private List<EnumMemberModel> members;

    public EnumModel(String enumName, Collection<String> enumMembers) {
        name = enumName;
        members = new ArrayList<>(enumMembers.size());
        for (String enumMember : enumMembers) {
           members.add(new EnumMemberModel(PlatformAndKeywordSanitizer.fixEnumValue(enumMember), enumMember));
        }
    }

}

