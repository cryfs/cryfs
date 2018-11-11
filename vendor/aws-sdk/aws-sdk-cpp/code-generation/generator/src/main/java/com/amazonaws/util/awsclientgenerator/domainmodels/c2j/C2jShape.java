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

package com.amazonaws.util.awsclientgenerator.domainmodels.c2j;

import com.google.gson.annotations.SerializedName;
import lombok.Data;

import java.util.List;
import java.util.Map;

@Data
public class C2jShape {
    private String type;
    private List<String> required;
    @SerializedName(value = "enum")
    private List<String> enums;
    private Map<String, C2jShapeMember> members;
    private C2jShapeMember member;
    private C2jShapeMember key;
    private C2jShapeMember value;
    private String max;
    private String min;
    private String documentation;
    private String locationName;
    private String payload;
    private boolean flattened;
    private C2jErrorInfo error;
    private boolean exception;
    private String timestampFormat;
    private boolean eventstream;
    private boolean event;
    private boolean sensitive;
}
