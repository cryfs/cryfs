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

#include <aws/s3/model/Destination.h>
#include <aws/core/utils/xml/XmlSerializer.h>
#include <aws/core/utils/StringUtils.h>
#include <aws/core/utils/memory/stl/AWSStringStream.h>

#include <utility>

using namespace Aws::Utils::Xml;
using namespace Aws::Utils;

namespace Aws
{
namespace S3
{
namespace Model
{

Destination::Destination() : 
    m_bucketHasBeenSet(false),
    m_accountHasBeenSet(false),
    m_storageClass(StorageClass::NOT_SET),
    m_storageClassHasBeenSet(false),
    m_accessControlTranslationHasBeenSet(false),
    m_encryptionConfigurationHasBeenSet(false)
{
}

Destination::Destination(const XmlNode& xmlNode) : 
    m_bucketHasBeenSet(false),
    m_accountHasBeenSet(false),
    m_storageClass(StorageClass::NOT_SET),
    m_storageClassHasBeenSet(false),
    m_accessControlTranslationHasBeenSet(false),
    m_encryptionConfigurationHasBeenSet(false)
{
  *this = xmlNode;
}

Destination& Destination::operator =(const XmlNode& xmlNode)
{
  XmlNode resultNode = xmlNode;

  if(!resultNode.IsNull())
  {
    XmlNode bucketNode = resultNode.FirstChild("Bucket");
    if(!bucketNode.IsNull())
    {
      m_bucket = StringUtils::Trim(bucketNode.GetText().c_str());
      m_bucketHasBeenSet = true;
    }
    XmlNode accountNode = resultNode.FirstChild("Account");
    if(!accountNode.IsNull())
    {
      m_account = StringUtils::Trim(accountNode.GetText().c_str());
      m_accountHasBeenSet = true;
    }
    XmlNode storageClassNode = resultNode.FirstChild("StorageClass");
    if(!storageClassNode.IsNull())
    {
      m_storageClass = StorageClassMapper::GetStorageClassForName(StringUtils::Trim(storageClassNode.GetText().c_str()).c_str());
      m_storageClassHasBeenSet = true;
    }
    XmlNode accessControlTranslationNode = resultNode.FirstChild("AccessControlTranslation");
    if(!accessControlTranslationNode.IsNull())
    {
      m_accessControlTranslation = accessControlTranslationNode;
      m_accessControlTranslationHasBeenSet = true;
    }
    XmlNode encryptionConfigurationNode = resultNode.FirstChild("EncryptionConfiguration");
    if(!encryptionConfigurationNode.IsNull())
    {
      m_encryptionConfiguration = encryptionConfigurationNode;
      m_encryptionConfigurationHasBeenSet = true;
    }
  }

  return *this;
}

void Destination::AddToNode(XmlNode& parentNode) const
{
  Aws::StringStream ss;
  if(m_bucketHasBeenSet)
  {
   XmlNode bucketNode = parentNode.CreateChildElement("Bucket");
   bucketNode.SetText(m_bucket);
  }

  if(m_accountHasBeenSet)
  {
   XmlNode accountNode = parentNode.CreateChildElement("Account");
   accountNode.SetText(m_account);
  }

  if(m_storageClassHasBeenSet)
  {
   XmlNode storageClassNode = parentNode.CreateChildElement("StorageClass");
   storageClassNode.SetText(StorageClassMapper::GetNameForStorageClass(m_storageClass));
  }

  if(m_accessControlTranslationHasBeenSet)
  {
   XmlNode accessControlTranslationNode = parentNode.CreateChildElement("AccessControlTranslation");
   m_accessControlTranslation.AddToNode(accessControlTranslationNode);
  }

  if(m_encryptionConfigurationHasBeenSet)
  {
   XmlNode encryptionConfigurationNode = parentNode.CreateChildElement("EncryptionConfiguration");
   m_encryptionConfiguration.AddToNode(encryptionConfigurationNode);
  }

}

} // namespace Model
} // namespace S3
} // namespace Aws
