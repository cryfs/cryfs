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

import lombok.Data;

import java.lang.*;
import java.util.List;

@Data
public class C2jOperation {
    private String name;
    private String authtype;
    private String authorizer;
    private C2jHttp http;
    private C2jShapeMember input;
    private C2jShapeMember output;
    private List<C2jError> errors;
    private String documentation;
    private boolean deprecated;
}
