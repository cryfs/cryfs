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

#include <aws/testing/platform/PlatformTesting.h>

#pragma warning(disable: 4996)
#include <windows.h>
#include <aws/core/utils/memory/stl/AWSStringStream.h>

namespace Aws
{
namespace Environment
{

int SetEnv(const char* name, const char* value, int overwrite)
{
    AWS_UNREFERENCED_PARAM(overwrite);

    std::stringstream ss;
    ss << name << "=" << value;
    return _putenv(ss.str().c_str());
}

int UnSetEnv(const char* name)
{
    return SetEnv(name, "", 1);
}

} // namespace Environment

namespace Testing
{
    void InitPlatformTest(Aws::SDKOptions& sdkOptions)
    {
        AWS_UNREFERENCED_PARAM(sdkOptions);
    }

    void ShutdownPlatformTest(Aws::SDKOptions& sdkOptions)
    {
        AWS_UNREFERENCED_PARAM(sdkOptions);
    }
    const char* GetDefaultWriteFolder()
    {
        return "";
    }
} // namespace Testing
} // namespace Aws
