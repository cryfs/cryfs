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

import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Data
public class Http {
    private static final Pattern URI_PARAM_PATTERN = Pattern.compile(".*\\{[\\w\\d]+\\}");

    private String method;
    private String requestUri;
    private String responseCode;

    public List<String> getRequestUriParts() {
        String sanitizedUri = requestUri.replace("+", "");
        return Arrays.asList(sanitizedUri.split("\\{[\\w\\d]+\\}"));
    }

    public List<String> getRequestParameters() {
        String sanitizedUri = requestUri.replace("+", "");
        String[] parts = sanitizedUri.split("/|\\?|&");
        List<String> paramList = new LinkedList<>();

        for (String part : parts) {
            Matcher matcher = URI_PARAM_PATTERN.matcher(part);

            if (matcher.matches()) {
               paramList.add(part.substring(part.indexOf("{") + 1, part.indexOf("}")));
            }
        }

        return paramList;
    }

    public List<String> splitUriPartIntoPathAndQuery(String requestUri) {
        String[] pathAndQuery = requestUri.split("\\?", -1);
        if (pathAndQuery.length != 2) {
            throw new IllegalArgumentException("Unexpected number of question marks in the request URI, one and only one is expected.");
        }
        pathAndQuery[1] = "?" + pathAndQuery[1];
        return Arrays.asList(pathAndQuery);
    }
}
