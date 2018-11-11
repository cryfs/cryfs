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

package com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.cpp;

import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Http;
import org.junit.Test;

import java.util.List;

import static org.junit.Assert.assertEquals;

public class HttpTest {

    @Test
    public void testParseHttpURIParts() {
        Http http = new Http();
        http.setRequestUri("/test/{var1}/test1/{var2}");

        List<String> parts = http.getRequestUriParts();
        assertEquals(2, parts.size());
        assertEquals("/test/", parts.get(0));
        assertEquals("/test1/", parts.get(1));

        http.setRequestUri("/test/{var1}/{var2}");

        parts = http.getRequestUriParts();
        assertEquals(2, parts.size());
        assertEquals("/test/", parts.get(0));
        assertEquals("/", parts.get(1));

        http.setRequestUri("/test/{var1}/test1/{var2+}?varParam={var3}");
        parts = http.getRequestUriParts();
        assertEquals(3, parts.size());
        assertEquals("/test/", parts.get(0));
        assertEquals("/test1/", parts.get(1));
        assertEquals("?varParam=", parts.get(2));

        http.setRequestUri("/");
        parts = http.getRequestUriParts();
        assertEquals(1, parts.size());
        assertEquals("/", parts.get(0));
    }

    @Test
    public void testParseHttpParameters() {
        Http http = new Http();
        http.setRequestUri("/test/{var1}/test1/{var2}");

        List<String> vars = http.getRequestParameters();
        assertEquals(2, vars.size());
        assertEquals("var1", vars.get(0));
        assertEquals("var2", vars.get(1));

        http.setRequestUri("/test/{var1}/{var2}");

        vars = http.getRequestParameters();
        assertEquals(2, vars.size());
        assertEquals("var1", vars.get(0));
        assertEquals("var2", vars.get(1));

        http.setRequestUri("/test/{var1}/test1/{var2+}?varParam={var3}&varParam2={var4}");
        vars = http.getRequestParameters();
        assertEquals(4, vars.size());
        assertEquals("var1", vars.get(0));
        assertEquals("var2", vars.get(1));
        assertEquals("var3", vars.get(2));
        assertEquals("var4", vars.get(3));

        http.setRequestUri("/");
        vars = http.getRequestParameters();
        assertEquals(0, vars.size());
    }

    @Test
    public void testSplitUriPartIntoPathAndQuery() {
        Http http = new Http();
        String requestUri = "/test?varParam=var";

        List<String> pathAndQuery = http.splitUriPartIntoPathAndQuery(requestUri);
        assertEquals(2, pathAndQuery.size());
        assertEquals("/test", pathAndQuery.get(0));
        assertEquals("?varParam=var", pathAndQuery.get(1));

        requestUri = "?varParam=var";
        pathAndQuery = http.splitUriPartIntoPathAndQuery(requestUri);
        assertEquals(2, pathAndQuery.size());
        assertEquals("", pathAndQuery.get(0));
        assertEquals("?varParam=var", pathAndQuery.get(1));

        requestUri = "/test?";
        pathAndQuery = http.splitUriPartIntoPathAndQuery(requestUri);
        assertEquals(2, pathAndQuery.size());
        assertEquals("/test", pathAndQuery.get(0));
        assertEquals("?", pathAndQuery.get(1));
    }

    @Test(expected = IllegalArgumentException.class)
    public void testNoQuestionMarkInRequestUri() {
        Http http = new Http();
        String requestUri = "/testvarParam=var";

        http.splitUriPartIntoPathAndQuery(requestUri);
    }

    @Test(expected = IllegalArgumentException.class)
    public void testMoreThanOneQuestionMarkInRequestUri() {
        Http http = new Http();
        String requestUri = "/test?test1?varParam=var";

        http.splitUriPartIntoPathAndQuery(requestUri);
    }
}
