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

import lombok.AccessLevel;
import lombok.Data;
import lombok.Getter;
import lombok.Setter;

import java.util.*;

@Data
public class ServiceModel {
    String version;
    String runtimeMajorVersion;
    String runtimeMajorVersionUpperBound;
    String runtimeMinorVersion;
    String namespace;
    Metadata metadata;
    String documentation;
    String licenseText;
    Map<String, Shape> shapes;
    Map<String, Operation> operations;
    Collection<Error> serviceErrors;

    @Getter(AccessLevel.PRIVATE)
    @Setter(AccessLevel.PRIVATE)
    Set<String> inputShapes = new HashSet<>();

    @Getter(AccessLevel.PRIVATE)
    @Setter(AccessLevel.PRIVATE)
    Set<String> outputShapes = new HashSet<>();

    public boolean hasStreamingRequestShapes() {
        return shapes.values().parallelStream().anyMatch(shape -> shape.isRequest() && shape.hasStreamMembers());
    }

}
