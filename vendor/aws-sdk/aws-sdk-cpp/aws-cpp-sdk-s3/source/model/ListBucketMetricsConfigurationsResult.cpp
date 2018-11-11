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

#include <aws/s3/model/ListBucketMetricsConfigurationsResult.h>
#include <aws/core/utils/xml/XmlSerializer.h>
#include <aws/core/AmazonWebServiceResult.h>
#include <aws/core/utils/StringUtils.h>

#include <utility>

using namespace Aws::S3::Model;
using namespace Aws::Utils::Xml;
using namespace Aws::Utils;
using namespace Aws;

ListBucketMetricsConfigurationsResult::ListBucketMetricsConfigurationsResult() : 
    m_isTruncated(false)
{
}

ListBucketMetricsConfigurationsResult::ListBucketMetricsConfigurationsResult(const Aws::AmazonWebServiceResult<XmlDocument>& result) : 
    m_isTruncated(false)
{
  *this = result;
}

ListBucketMetricsConfigurationsResult& ListBucketMetricsConfigurationsResult::operator =(const Aws::AmazonWebServiceResult<XmlDocument>& result)
{
  const XmlDocument& xmlDocument = result.GetPayload();
  XmlNode resultNode = xmlDocument.GetRootElement();

  if(!resultNode.IsNull())
  {
    XmlNode isTruncatedNode = resultNode.FirstChild("IsTruncated");
    if(!isTruncatedNode.IsNull())
    {
      m_isTruncated = StringUtils::ConvertToBool(StringUtils::Trim(isTruncatedNode.GetText().c_str()).c_str());
    }
    XmlNode continuationTokenNode = resultNode.FirstChild("ContinuationToken");
    if(!continuationTokenNode.IsNull())
    {
      m_continuationToken = StringUtils::Trim(continuationTokenNode.GetText().c_str());
    }
    XmlNode nextContinuationTokenNode = resultNode.FirstChild("NextContinuationToken");
    if(!nextContinuationTokenNode.IsNull())
    {
      m_nextContinuationToken = StringUtils::Trim(nextContinuationTokenNode.GetText().c_str());
    }
    XmlNode metricsConfigurationListNode = resultNode.FirstChild("MetricsConfiguration");
    if(!metricsConfigurationListNode.IsNull())
    {
      XmlNode metricsConfigurationMember = metricsConfigurationListNode;
      while(!metricsConfigurationMember.IsNull())
      {
        m_metricsConfigurationList.push_back(metricsConfigurationMember);
        metricsConfigurationMember = metricsConfigurationMember.NextNode("MetricsConfiguration");
      }

    }
  }

  return *this;
}
