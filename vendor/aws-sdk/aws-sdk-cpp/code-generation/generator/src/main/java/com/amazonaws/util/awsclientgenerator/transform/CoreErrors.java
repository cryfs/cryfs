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

package com.amazonaws.util.awsclientgenerator.transform;

import java.util.HashSet;
import java.util.Set;

public class CoreErrors
{
    public static final Set<String> VARIANTS = new HashSet<>();

    static {
        VARIANTS.add("IncompleteSignature");
        VARIANTS.add("IncompleteSignatureException");
        VARIANTS.add("InternalFailure");
        VARIANTS.add("InternalFailureException");
        VARIANTS.add("InvalidAction");
        VARIANTS.add("InvalidActionException");
        VARIANTS.add("InvalidClientTokenId");
        VARIANTS.add("InvalidClientTokenIdException");
        VARIANTS.add("InvalidParameterCombination");
        VARIANTS.add("InvalidParameterCombinationException");
        VARIANTS.add("InvalidParameterValue");
        VARIANTS.add("InvalidParameterValueException");
        VARIANTS.add("InvalidQueryParameter");
        VARIANTS.add("InvalidQueryParameterException");
        VARIANTS.add("MalformedQueryString");
        VARIANTS.add("MalformedQueryStringException");
        VARIANTS.add("MissingAction");
        VARIANTS.add("MissingActionException");
        VARIANTS.add("MissingAuthenticationToken");
        VARIANTS.add("MissingAuthenticationTokenException");
        VARIANTS.add("MissingParameter");
        VARIANTS.add("MissingParameterException");
        VARIANTS.add("OptInRequired");
        VARIANTS.add("RequestExpired");
        VARIANTS.add("RequestExpiredException");
        VARIANTS.add("ServiceUnavailable");
        VARIANTS.add("ServiceUnavailableException");
        VARIANTS.add("ServiceUnavailableError");
        VARIANTS.add("Throttling");
        VARIANTS.add("ThrottlingException");
        VARIANTS.add("ValidationError");
        VARIANTS.add("ValidationErrorException");
        VARIANTS.add("ValidationException");
        VARIANTS.add("AccessDenied");
        VARIANTS.add("AccessDeniedException");
        VARIANTS.add("ResourceNotFound");
        VARIANTS.add("ResourceNotFoundException");
        VARIANTS.add("UnrecognizedClient");
        VARIANTS.add("UnrecognizedClientException");
        VARIANTS.add("InternalServerError");
        VARIANTS.add("SlowDown");
        VARIANTS.add("SlowDownException");
        VARIANTS.add("RequestTimeTooSkewed");
        VARIANTS.add("RequestTimeTooSkewedException");
        VARIANTS.add("InvalidSignature");
        VARIANTS.add("InvalidSignatureException");
        VARIANTS.add("SignatureDoesNotMatch");
        VARIANTS.add("SignatureDoesNotMatchException");
        VARIANTS.add("InvalidAccessKeyId");
        VARIANTS.add("InvalidAccessKeyIdException");
    }

}
