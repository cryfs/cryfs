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

#include <aws/s3/model/CSVInput.h>
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

CSVInput::CSVInput() : 
    m_fileHeaderInfo(FileHeaderInfo::NOT_SET),
    m_fileHeaderInfoHasBeenSet(false),
    m_commentsHasBeenSet(false),
    m_quoteEscapeCharacterHasBeenSet(false),
    m_recordDelimiterHasBeenSet(false),
    m_fieldDelimiterHasBeenSet(false),
    m_quoteCharacterHasBeenSet(false),
    m_allowQuotedRecordDelimiter(false),
    m_allowQuotedRecordDelimiterHasBeenSet(false)
{
}

CSVInput::CSVInput(const XmlNode& xmlNode) : 
    m_fileHeaderInfo(FileHeaderInfo::NOT_SET),
    m_fileHeaderInfoHasBeenSet(false),
    m_commentsHasBeenSet(false),
    m_quoteEscapeCharacterHasBeenSet(false),
    m_recordDelimiterHasBeenSet(false),
    m_fieldDelimiterHasBeenSet(false),
    m_quoteCharacterHasBeenSet(false),
    m_allowQuotedRecordDelimiter(false),
    m_allowQuotedRecordDelimiterHasBeenSet(false)
{
  *this = xmlNode;
}

CSVInput& CSVInput::operator =(const XmlNode& xmlNode)
{
  XmlNode resultNode = xmlNode;

  if(!resultNode.IsNull())
  {
    XmlNode fileHeaderInfoNode = resultNode.FirstChild("FileHeaderInfo");
    if(!fileHeaderInfoNode.IsNull())
    {
      m_fileHeaderInfo = FileHeaderInfoMapper::GetFileHeaderInfoForName(StringUtils::Trim(fileHeaderInfoNode.GetText().c_str()).c_str());
      m_fileHeaderInfoHasBeenSet = true;
    }
    XmlNode commentsNode = resultNode.FirstChild("Comments");
    if(!commentsNode.IsNull())
    {
      m_comments = StringUtils::Trim(commentsNode.GetText().c_str());
      m_commentsHasBeenSet = true;
    }
    XmlNode quoteEscapeCharacterNode = resultNode.FirstChild("QuoteEscapeCharacter");
    if(!quoteEscapeCharacterNode.IsNull())
    {
      m_quoteEscapeCharacter = StringUtils::Trim(quoteEscapeCharacterNode.GetText().c_str());
      m_quoteEscapeCharacterHasBeenSet = true;
    }
    XmlNode recordDelimiterNode = resultNode.FirstChild("RecordDelimiter");
    if(!recordDelimiterNode.IsNull())
    {
      m_recordDelimiter = StringUtils::Trim(recordDelimiterNode.GetText().c_str());
      m_recordDelimiterHasBeenSet = true;
    }
    XmlNode fieldDelimiterNode = resultNode.FirstChild("FieldDelimiter");
    if(!fieldDelimiterNode.IsNull())
    {
      m_fieldDelimiter = StringUtils::Trim(fieldDelimiterNode.GetText().c_str());
      m_fieldDelimiterHasBeenSet = true;
    }
    XmlNode quoteCharacterNode = resultNode.FirstChild("QuoteCharacter");
    if(!quoteCharacterNode.IsNull())
    {
      m_quoteCharacter = StringUtils::Trim(quoteCharacterNode.GetText().c_str());
      m_quoteCharacterHasBeenSet = true;
    }
    XmlNode allowQuotedRecordDelimiterNode = resultNode.FirstChild("AllowQuotedRecordDelimiter");
    if(!allowQuotedRecordDelimiterNode.IsNull())
    {
      m_allowQuotedRecordDelimiter = StringUtils::ConvertToBool(StringUtils::Trim(allowQuotedRecordDelimiterNode.GetText().c_str()).c_str());
      m_allowQuotedRecordDelimiterHasBeenSet = true;
    }
  }

  return *this;
}

void CSVInput::AddToNode(XmlNode& parentNode) const
{
  Aws::StringStream ss;
  if(m_fileHeaderInfoHasBeenSet)
  {
   XmlNode fileHeaderInfoNode = parentNode.CreateChildElement("FileHeaderInfo");
   fileHeaderInfoNode.SetText(FileHeaderInfoMapper::GetNameForFileHeaderInfo(m_fileHeaderInfo));
  }

  if(m_commentsHasBeenSet)
  {
   XmlNode commentsNode = parentNode.CreateChildElement("Comments");
   commentsNode.SetText(m_comments);
  }

  if(m_quoteEscapeCharacterHasBeenSet)
  {
   XmlNode quoteEscapeCharacterNode = parentNode.CreateChildElement("QuoteEscapeCharacter");
   quoteEscapeCharacterNode.SetText(m_quoteEscapeCharacter);
  }

  if(m_recordDelimiterHasBeenSet)
  {
   XmlNode recordDelimiterNode = parentNode.CreateChildElement("RecordDelimiter");
   recordDelimiterNode.SetText(m_recordDelimiter);
  }

  if(m_fieldDelimiterHasBeenSet)
  {
   XmlNode fieldDelimiterNode = parentNode.CreateChildElement("FieldDelimiter");
   fieldDelimiterNode.SetText(m_fieldDelimiter);
  }

  if(m_quoteCharacterHasBeenSet)
  {
   XmlNode quoteCharacterNode = parentNode.CreateChildElement("QuoteCharacter");
   quoteCharacterNode.SetText(m_quoteCharacter);
  }

  if(m_allowQuotedRecordDelimiterHasBeenSet)
  {
   XmlNode allowQuotedRecordDelimiterNode = parentNode.CreateChildElement("AllowQuotedRecordDelimiter");
   ss << std::boolalpha << m_allowQuotedRecordDelimiter;
   allowQuotedRecordDelimiterNode.SetText(ss.str());
   ss.str("");
  }

}

} // namespace Model
} // namespace S3
} // namespace Aws
