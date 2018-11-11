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

#pragma once
#include <aws/s3/S3_EXPORTS.h>
#include <aws/core/utils/memory/stl/AWSString.h>
#include <aws/core/utils/memory/stl/AWSVector.h>
#include <aws/s3/model/ReplicationRule.h>
#include <utility>

namespace Aws
{
namespace Utils
{
namespace Xml
{
  class XmlNode;
} // namespace Xml
} // namespace Utils
namespace S3
{
namespace Model
{

  /**
   * <p>Container for replication rules. You can add as many as 1,000 rules. Total
   * replication configuration size can be up to 2 MB.</p><p><h3>See Also:</h3>   <a
   * href="http://docs.aws.amazon.com/goto/WebAPI/s3-2006-03-01/ReplicationConfiguration">AWS
   * API Reference</a></p>
   */
  class AWS_S3_API ReplicationConfiguration
  {
  public:
    ReplicationConfiguration();
    ReplicationConfiguration(const Aws::Utils::Xml::XmlNode& xmlNode);
    ReplicationConfiguration& operator=(const Aws::Utils::Xml::XmlNode& xmlNode);

    void AddToNode(Aws::Utils::Xml::XmlNode& parentNode) const;


    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline const Aws::String& GetRole() const{ return m_role; }

    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline void SetRole(const Aws::String& value) { m_roleHasBeenSet = true; m_role = value; }

    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline void SetRole(Aws::String&& value) { m_roleHasBeenSet = true; m_role = std::move(value); }

    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline void SetRole(const char* value) { m_roleHasBeenSet = true; m_role.assign(value); }

    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline ReplicationConfiguration& WithRole(const Aws::String& value) { SetRole(value); return *this;}

    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline ReplicationConfiguration& WithRole(Aws::String&& value) { SetRole(std::move(value)); return *this;}

    /**
     * <p>Amazon Resource Name (ARN) of an IAM role for Amazon S3 to assume when
     * replicating the objects.</p>
     */
    inline ReplicationConfiguration& WithRole(const char* value) { SetRole(value); return *this;}


    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline const Aws::Vector<ReplicationRule>& GetRules() const{ return m_rules; }

    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline void SetRules(const Aws::Vector<ReplicationRule>& value) { m_rulesHasBeenSet = true; m_rules = value; }

    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline void SetRules(Aws::Vector<ReplicationRule>&& value) { m_rulesHasBeenSet = true; m_rules = std::move(value); }

    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline ReplicationConfiguration& WithRules(const Aws::Vector<ReplicationRule>& value) { SetRules(value); return *this;}

    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline ReplicationConfiguration& WithRules(Aws::Vector<ReplicationRule>&& value) { SetRules(std::move(value)); return *this;}

    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline ReplicationConfiguration& AddRules(const ReplicationRule& value) { m_rulesHasBeenSet = true; m_rules.push_back(value); return *this; }

    /**
     * <p>Container for one or more replication rules. Replication configuration must
     * have at least one rule and can contain up to 1,000 rules. </p>
     */
    inline ReplicationConfiguration& AddRules(ReplicationRule&& value) { m_rulesHasBeenSet = true; m_rules.push_back(std::move(value)); return *this; }

  private:

    Aws::String m_role;
    bool m_roleHasBeenSet;

    Aws::Vector<ReplicationRule> m_rules;
    bool m_rulesHasBeenSet;
  };

} // namespace Model
} // namespace S3
} // namespace Aws
