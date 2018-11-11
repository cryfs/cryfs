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

import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.ServiceModel;
import com.amazonaws.util.awsclientgenerator.domainmodels.codegeneration.Shape;
import lombok.Data;

import java.util.Set;

@Data
public class CppShapeInformation {
   private final Shape shape;
   private final ServiceModel serviceModel;
   private final String className;
   private final String jsonType = "Aws::Utils::Json::JsonValue";
   private final String jsonViewType = "Aws::Utils::Json::JsonView";
   private final String xmlDocType = "Aws::Utils::Xml::XmlDocument";
   private final String xmlNodeType = "Aws::Utils::Xml::XmlNode";
   private final String exportValue;
   private final String cppType;
   private final Set<String> headerIncludes;
   private final Set<String> sourceIncludes;
   private final String baseClass;
   private final String requestContentType;

   public CppShapeInformation(final Shape shape, final ServiceModel serviceModel) {
       this.shape = shape;
       this.serviceModel = serviceModel;
       className = shape.getName();
       exportValue = CppViewHelper.computeExportValue(serviceModel.getMetadata().getClassNamePrefix());
       cppType = CppViewHelper.computeCppType(shape);
       headerIncludes = CppViewHelper.computeHeaderIncludes(serviceModel.getMetadata().getProjectName(), shape);
       sourceIncludes = CppViewHelper.computeSourceIncludes(shape);
       baseClass = CppViewHelper.computeBaseClass(serviceModel.getMetadata().getClassNamePrefix(), shape);
       requestContentType = CppViewHelper.computeRequestContentType(serviceModel.getMetadata());
   }
}
