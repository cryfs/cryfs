/*
* Copyright 2010-2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
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

#include <limits>
#include <aws/core/client/AWSClient.h>
#include <aws/core/client/AWSError.h>
#include <aws/core/client/ClientConfiguration.h>
#include <aws/core/client/DefaultRetryStrategy.h>
#include <aws/core/AmazonWebServiceRequest.h>
#include <aws/core/auth/AWSAuthSigner.h>
#include <aws/core/auth/AWSCredentialsProvider.h>
#include <aws/core/http/standard/StandardHttpRequest.h>
#include <aws/core/http/standard/StandardHttpResponse.h>
#include <aws/core/utils/memory/stl/AWSAllocator.h>
#include <aws/core/utils/memory/stl/AWSStringStream.h>
#include <aws/core/utils/Outcome.h>
#include <aws/testing/mocks/http/MockHttpClient.h>

using namespace Aws::Client;
using namespace Aws::Http::Standard;
using namespace Aws::Http;
using namespace Aws;

class AmazonWebServiceRequestMock : public AmazonWebServiceRequest
{
public:
    AmazonWebServiceRequestMock() : m_shouldComputeMd5(false) { }
    std::shared_ptr<Aws::IOStream> GetBody() const override { return m_body; }
    void SetBody(const std::shared_ptr<Aws::IOStream>& body) { m_body = body; }
    HeaderValueCollection GetHeaders() const override { return m_headers; }
    void SetHeaders(const HeaderValueCollection& value) { m_headers = value; }
    bool ShouldComputeContentMd5() const override { return m_shouldComputeMd5; }
    void SetComputeContentMd5(bool value) { m_shouldComputeMd5 = value; }
    virtual const char* GetServiceRequestName() const override { return "AmazonWebServiceRequestMock"; }

private:
    std::shared_ptr<Aws::IOStream> m_body;
    HeaderValueCollection m_headers;
    bool m_shouldComputeMd5;
};

class CountedRetryStrategy : public DefaultRetryStrategy
{
public:
    CountedRetryStrategy() : m_attemptedRetries(0), m_maxRetries(std::numeric_limits<int>::max()) {}
    CountedRetryStrategy(int maxRetires) : m_attemptedRetries(0), m_maxRetries(maxRetires <= 0 ? std::numeric_limits<int>::max() : maxRetires) {}

    bool ShouldRetry(const AWSError<CoreErrors>& error, long attemptedRetries) const override
    {
        if (attemptedRetries >= m_maxRetries)
        {
            return false;
        }
        if(DefaultRetryStrategy::ShouldRetry(error, attemptedRetries)) 
        {
            m_attemptedRetries = attemptedRetries + 1;
            return true;
        }
        return false;
    }
    int GetAttemptedRetriesCount() { return m_attemptedRetries; }
    void ResetAttemptedRetriesCount() { m_attemptedRetries = 0; }
private:
    mutable int m_attemptedRetries;
    int m_maxRetries;
};

class MockAWSClient : AWSClient
{
public:
    MockAWSClient(const ClientConfiguration& config) : AWSClient(config, 
            Aws::MakeShared<AWSAuthV4Signer>("MockAWSClient", 
                Aws::MakeShared<Aws::Auth::SimpleAWSCredentialsProvider>("MockAWSClient", GetMockAccessKey(), 
                    GetMockSecretAccessKey()), "service", config.region.empty() ? Aws::Region::US_EAST_1 : config.region), nullptr) ,
        m_countedRetryStrategy(std::static_pointer_cast<CountedRetryStrategy>(config.retryStrategy)) { }

    Aws::Client::HttpResponseOutcome MakeRequest(const AmazonWebServiceRequest& request)
    {
        m_countedRetryStrategy->ResetAttemptedRetriesCount();
        const URI uri("domain.com/something");
        const auto method = HttpMethod::HTTP_GET;
        HttpResponseOutcome httpOutcome(AWSClient::AttemptExhaustively(uri, request, method, Aws::Auth::SIGV4_SIGNER));
        return httpOutcome;
    }

    inline static const char* GetMockAccessKey() { return "AKIDEXAMPLE"; }
    inline static const char* GetMockSecretAccessKey() { return "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY"; }

    int GetRequestAttemptedRetries()
    {
        return m_countedRetryStrategy->GetAttemptedRetriesCount();
    }

    inline const char* GetServiceClientName() const override { return "MockAWSClient"; }

protected:
    std::shared_ptr<CountedRetryStrategy> m_countedRetryStrategy;
    AWSError<CoreErrors> BuildAWSError(const std::shared_ptr<HttpResponse>& response) const override
    {
        if (!response)
        {
            auto err = AWSError<CoreErrors>(CoreErrors::NETWORK_CONNECTION, "", "Unable to connect to endpoint", true);
            err.SetResponseCode(HttpResponseCode::INTERNAL_SERVER_ERROR);
            return err;
        }
        auto err = AWSError<CoreErrors>(CoreErrors::INVALID_ACTION, false);
        err.SetResponseHeaders(response->GetHeaders());
        err.SetResponseCode(response->GetResponseCode());
        return err;
    }
};
