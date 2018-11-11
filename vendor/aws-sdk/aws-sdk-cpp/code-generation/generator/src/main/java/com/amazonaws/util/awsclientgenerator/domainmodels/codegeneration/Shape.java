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

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.HashSet;
import java.util.Optional;

@Data
public class Shape {
    private String name;
    private String type;
    private List<String> enumValues;
    private HashSet<String> referencedBy;
    private Map<String, ShapeMember> members;
    private ShapeMember listMember;
    private ShapeMember mapKey;
    private ShapeMember mapValue;
    private ShapeMember customizedQuery;
    private String max;
    private String min;
    private String documentation;
    private String locationName;
    private String payload;
    private String xmlNamespace;
    private boolean isRequest;
    private boolean isResult;
    private boolean isReferenced;
    private boolean flattened;
    private boolean computeContentMd5;
    private boolean supportsPresigning;
    private boolean signBody;
    private String signerName;
    private String timestampFormat;
    private boolean sensitive;

    public boolean isMap() {
        return "map".equals(type.toLowerCase());
    }

    public boolean isList() {
        return "list".equals(type.toLowerCase());
    }

    public boolean isStructure() {
        return "structure".equals(type.toLowerCase());
    }

    public boolean isDouble() { return "double".equals(type.toLowerCase()); }

    public boolean isString() {
        return "string".equals(type.toLowerCase()) && !isEnum();
    }

    public boolean isTimeStamp() {
        return "timestamp".equals(type.toLowerCase());
    }

    public boolean isEnum() {
        return enumValues != null && enumValues.size() > 0;
    }

    public boolean isBlob() {
        return "blob".equals(type.toLowerCase());
    }

    public boolean isBoolean() {
        return "boolean".equals(type.toLowerCase());
    }

    public boolean isPrimitive() {
        return !isMap() && !isList() && !isStructure() && !isString() && !isEnum() && !isBlob() && !isTimeStamp();
    }

    public boolean isMemberRequired(String member) {
        ShapeMember shapeMember = members.get(member);
        return shapeMember != null && members.get(member).isRequired();
    }

    public boolean hasHeaderMembers() {
      return members.values().parallelStream().anyMatch(member -> member.isUsedForHeader());
    }

    public boolean hasStatusCodeMembers() {
      return members.values().parallelStream().anyMatch(member -> member.isUsedForHttpStatusCode());
    }

    public boolean hasStreamMembers() {
      if (members == null) return false;
      return members.values().parallelStream()
              .anyMatch(member -> member.isStreaming()) || (payload != null && members.get(payload) != null && !members.get(payload).getShape().isStructure() && !members.get(payload).getShape().isList());
    }

    public boolean hasPayloadMembers() {
       return members.values().parallelStream().anyMatch(member -> !member.isUsedForHttpStatusCode() && !member.isUsedForHeader()
               && !member.isStreaming() && !member.isUsedForUri() && !member.isUsedForQueryString());
    }

    public boolean hasQueryStringMembers() {
        return members.values().parallelStream().anyMatch(member -> member.getLocation() != null && member.getLocation().equalsIgnoreCase("querystring"));
    }

    public boolean hasBlobMembers() {
        return members.values().parallelStream().anyMatch(member -> member.getShape().isBlob());
    }

    public ShapeMember getMemberByLocationName(String locationName) {
        Optional<ShapeMember> found =
                members.values().parallelStream().filter(member -> member.getLocationName() != null && locationName.equals(member.getLocationName())).findFirst();

        if(found.isPresent()) {
            return found.get();
        }

        return null;

    }

    public String getMemberNameByLocationName(String locationName) {
        Optional<Map.Entry<String, ShapeMember>> found =
                members.entrySet().parallelStream().filter(member ->
                        locationName.equals(member.getValue().getLocationName())).findFirst();

        if(found.isPresent()) {
            return found.get().getKey();
        }

        return null;
    }

    public Map<String, ShapeMember> getQueryStringMembers() {
        Map<String, ShapeMember> queryStringMembers = new LinkedHashMap<>();
        for(Map.Entry<String, ShapeMember> entry : members.entrySet()) {
            if(entry.getValue().getLocation() != null && entry.getValue().getLocation().equalsIgnoreCase("querystring")){
               queryStringMembers.put(entry.getKey(), entry.getValue());
            }
        }

        return queryStringMembers;
    }

    public void RemoveMember(String memberName) {
        members.remove(memberName);
    }

    @Override
    public String toString() {
      if(name != null && type != null) {
        return String.format("Shape: Name %s Type %s, flattened %b", name, type, flattened);
      }

      return "null";
    }
}
