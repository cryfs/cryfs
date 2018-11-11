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

import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Error;
import com.google.common.base.CaseFormat;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;

public class ErrorFormatter {

    public List<String> formatErrorConstNames(Collection<Error> errors) {
        List<String> formattedErrors = new ArrayList<>();
        for (Error error : errors) {
            formattedErrors.add(formatErrorConstName(error.getName()));
        }
        Collections.sort(formattedErrors);
        return formattedErrors;
    }

    public static String formatErrorConstName(String errorName) {
        String upper = CaseFormat.UPPER_CAMEL.to(CaseFormat.UPPER_UNDERSCORE, errorName.replaceAll("\\.", "_"));
        if (upper.endsWith("_ERROR")) {
            upper = upper.substring(0, upper.length() - "_ERROR".length());
        }
        if (upper.endsWith("_EXCEPTION")) {
            upper = upper.substring(0, upper.length() - "_EXCEPTION".length());
        }
        return upper;
    }
}
