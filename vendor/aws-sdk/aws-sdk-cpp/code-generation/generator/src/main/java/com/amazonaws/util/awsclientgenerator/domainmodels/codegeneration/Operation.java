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

package com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration;

import lombok.Data;

import java.util.List;

@Data
public class Operation {
    private String name;
    private Http http;
    private ShapeMember request;
    private ShapeMember result;
    private List<Error> errors;
    private String documentation;
    private boolean supportsPresigning;
    private boolean virtualAddressAllowed;
    private String virtualAddressMemberName;
    private String authtype;
    private String authorizer;
}
