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

package com.amazonaws.util.awsclientgenerator.generators;

import com.amazonaws.util.awsclientgenerator.domainmodels.SdkFileEntry;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;

/**
 * Abstract class for generating AWS Client Code. All generators should implement this interface
 */
public interface ClientGenerator {

    /**
     * Generates all source files for a service based on a filled in service model
     *
     * @param serviceModel Service Model to use in generation.
     * @return
     */
    SdkFileEntry[] generateSourceFiles(final ServiceModel serviceModel) throws Exception;

}