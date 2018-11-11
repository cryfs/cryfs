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

package com.amazonaws.util.awsclientgenerator.config;

import com.amazonaws.util.awsclientgenerator.SdkSpec;
import com.amazonaws.util.awsclientgenerator.config.exceptions.GeneratorNotImplementedException;
import com.amazonaws.util.awsclientgenerator.generators.ClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.JsonCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.QueryCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.RestXmlCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.apigateway.APIGatewayRestJsonCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.cloudfront.CloudfrontCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.dynamodb.DynamoDBJsonCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.ec2.Ec2CppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.glacier.GlacierRestJsonCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.iam.IamCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.lambda.LambdaRestJsonCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.machinelearning.MachineLearningJsonCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.polly.PollyCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.r53.Route53CppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.rds.RDSCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.s3.S3RestXmlCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.sqs.SQSQueryXmlCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.budgets.BudgetsCppClientGenerator;
import com.amazonaws.util.awsclientgenerator.generators.cpp.waf.WafCppClientGenerator;

import java.util.HashMap;
import java.util.Map;

public class ServiceGeneratorConfig {
    static final Map<String, ClientGenerator> LANGUAGE_PROTOCOL_DEFAULT_MAPPING = new HashMap<>();
    static final Map<String, ClientGenerator> SPEC_OVERRIDE_MAPPING = new HashMap<>();

    static {
        try {
            LANGUAGE_PROTOCOL_DEFAULT_MAPPING.put("cpp-json", new JsonCppClientGenerator());
            LANGUAGE_PROTOCOL_DEFAULT_MAPPING.put("cpp-rest-json", new JsonCppClientGenerator());
            LANGUAGE_PROTOCOL_DEFAULT_MAPPING.put("cpp-application-json", new JsonCppClientGenerator());
            LANGUAGE_PROTOCOL_DEFAULT_MAPPING.put("cpp-rest-xml", new RestXmlCppClientGenerator());
            LANGUAGE_PROTOCOL_DEFAULT_MAPPING.put("cpp-query", new QueryCppClientGenerator());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    static {
        try {
            SPEC_OVERRIDE_MAPPING.put("cpp-dynamodb", new DynamoDBJsonCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-glacier", new GlacierRestJsonCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-lambda", new LambdaRestJsonCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-sqs", new SQSQueryXmlCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-s3", new S3RestXmlCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-iam", new IamCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-cloudfront", new CloudfrontCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-ec2", new Ec2CppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-apigateway", new APIGatewayRestJsonCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-machinelearning", new MachineLearningJsonCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-route53", new Route53CppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-budgets", new BudgetsCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-waf", new WafCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-polly", new PollyCppClientGenerator());
            SPEC_OVERRIDE_MAPPING.put("cpp-rds", new RDSCppClientGenerator());

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public static ClientGenerator findGenerator(final SdkSpec spec, final String protocol)
            throws GeneratorNotImplementedException {

        ClientGenerator generator = SPEC_OVERRIDE_MAPPING.get(String.format("%s-%s", spec.getLanguageBinding(), spec.getServiceName()));

        if (generator == null) {
            generator = LANGUAGE_PROTOCOL_DEFAULT_MAPPING.get(spec.getLanguageBinding() + "-" + protocol);
        }

        if(generator == null) {
           throw new GeneratorNotImplementedException(
                   String.format("No generator for Spec: %s protocol: %s", spec, protocol));
        }

        return generator;
    }

}
