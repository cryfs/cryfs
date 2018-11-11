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

package com.amazonaws.util.awsclientgenerator;

import com.amazonaws.util.awsclientgenerator.config.exceptions.GeneratorNotImplementedException;
import com.amazonaws.util.awsclientgenerator.generators.DirectFromC2jGenerator;
import com.amazonaws.util.awsclientgenerator.generators.MainClientGenerator;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;

public class main {

    static final String FILE_NAME = "inputfile";
    static final String ARBITRARY_OPTION = "arbitrary";
    static final String LANGUAGE_BINDING_OPTION = "language-binding";
    static final String SERVICE_OPTION = "service";
    static final String NAMESPACE = "namespace";
    static final String LICENSE_TEXT = "license-text";
    static final String STANDALONE_OPTION = "standlone";

    public static void main(String[] args) throws IOException {

        if (args.length == 0 || getOptionName(args[0]).equals("help")) {
            printHelp();
            return;
        }

        Map<String, String> argPairs = getArgOptionPairs(args);
        MainClientGenerator generator = new MainClientGenerator();

        String arbitraryJson = null;

        //At this point we want to read the c2j from std in.
        //e.g. cat /home/henso/someC2jFile.normal.json | AWSClientGenerator --service myService --language-binding cpp or
        //AWSClientGenerator --service myService --language-binding cpp < /home/henso/someC2jFile.normal.json
        if (argPairs.containsKey(ARBITRARY_OPTION) || argPairs.containsKey(FILE_NAME)) {
            if (!argPairs.containsKey(LANGUAGE_BINDING_OPTION) || argPairs.get(LANGUAGE_BINDING_OPTION).isEmpty()) {
                System.out.println("Error: A language binding must be specified with the --arbitrary option.");
                return;
            }
            if (!argPairs.containsKey(SERVICE_OPTION) || argPairs.get(SERVICE_OPTION).isEmpty()) {
                System.out.println("Error: A service name must be specified with the --arbitrary option.");
                return;
            }
            String namespace = "Aws";
            if (argPairs.containsKey(NAMESPACE) && !argPairs.get(NAMESPACE).isEmpty()) {
                namespace = argPairs.get(NAMESPACE);
            }
            String licenseText = "";
            if (argPairs.containsKey(LICENSE_TEXT) && !argPairs.get(LICENSE_TEXT).isEmpty()) {
                licenseText = argPairs.get(LICENSE_TEXT);
            }
            boolean generateStandalonePakckage = argPairs.containsKey(STANDALONE_OPTION);
            String languageBinding = argPairs.get(LANGUAGE_BINDING_OPTION);
            String serviceName = argPairs.get(SERVICE_OPTION);

            //read from the piped input
            try (InputStream stream = getInputStreamReader(argPairs)) {
                StringBuilder stringBuilder = new StringBuilder();

                byte[] buffer = new byte[1024];
                while (stream.read(buffer) > 0) {
                    stringBuilder.append(new String(buffer, StandardCharsets.UTF_8));
                }

                arbitraryJson = stringBuilder.toString();
            }

            if (arbitraryJson != null && arbitraryJson.length() > 0) {
                DirectFromC2jGenerator directFromC2jGenerator = new DirectFromC2jGenerator(generator);
                try {
                    File outputLib = directFromC2jGenerator.generateSourceFromJson(arbitraryJson,
                            languageBinding,
                            serviceName,
                            namespace,
                            licenseText,
                            generateStandalonePakckage);
                    System.out.println(outputLib.getAbsolutePath());
                } catch (GeneratorNotImplementedException e) {
                    e.printStackTrace();
                } catch (Exception e) {
                    e.printStackTrace();
                }
            } else {
                System.out.println("You must supply standard input if you specify the --arbitrary option.");

            }
            return;
        }

        printHelp();

    }

    private static InputStream getInputStreamReader(Map<String, String> argsMap) throws FileNotFoundException, UnsupportedEncodingException {
        if (argsMap.containsKey(FILE_NAME)) {
            return new FileInputStream(argsMap.get(FILE_NAME));
        }
        return System.in;
    }

    private static void printHelp() {
        System.out.println("Syntax: AWSClientGenerator <options>");
        System.out.println("Example: cat /home/henso/someC2jFile.normal.json | AWSClientGenerator --service myService --language-binding cpp --arbitrary");
        System.out.println("\tOptions:");
        System.out.println("\t\t--help  Print this message");
        System.out.println("\t\t--arbitrary Reads the Coral To Json Doc from the std input. You must specify --language-binding and --service with this argument.");
        System.out.println("\t\t--language-binding  language binding to generate sdk for. If this is specified you must specify service and version arguments or you must specify --arbitrary");
        System.out.println("\t\t--service service to generate service for. If this is specified, you must specify version and language-binding");
        System.out.println("\t\t--version version of service to generate sdk for. If this is specified, you must specify language-binding and service.");
        System.out.println("\t\t  If you generate a specific SDK, the output will be the file where the sdk is stored in zip format");
    }

    private static String getOptionName(String optionStr) {
        if (optionStr.contains("--")) {
            return optionStr.substring(2).toLowerCase();
        }

        return "";
    }

    private static boolean isOption(String str) {
        return !getOptionName(str).isEmpty();
    }

    /**
     * If we have option pairs (e.g. --key value) then this will put them into the map as key to value. If the option
     * does not take an argument, then the option is a key but the value will be empty.
     *
     * @param args cli args to parse.
     * @return map of passed options and their arguments if they exist.
     */
    private static Map<String, String> getArgOptionPairs(String[] args) {
        Map<String, String> argsPairs = new HashMap<>();
        for (int i = 0; i < args.length; ) {
            String key = "", value = "";
            if (isOption(args[i])) {
                key = getOptionName(args[i]);
                ++i;

                if (i < args.length && !isOption(args[i])) {
                    value = args[i];
                    ++i;
                }
                argsPairs.put(key, value);
            } else {
                ++i;
            }
        }

        return argsPairs;
    }
}
