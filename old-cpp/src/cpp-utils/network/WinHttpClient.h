#pragma once
#ifndef MESSMER_CPPUTILS_NETWORK_WINHTTPCLIENT_HPP
#define MESSMER_CPPUTILS_NETWORK_WINHTTPCLIENT_HPP

#if defined(_MSC_VER)

#include "HttpClient.h"
#include "../macros.h"
#include "../pointer/unique_ref.h"

namespace cpputils {

	class WinHttpSession;

	class WinHttpClient final : public HttpClient {
	public:
		WinHttpClient();
		~WinHttpClient();

		std::string get(const std::string &url, boost::optional<long> timeoutMsec = boost::none) override;

	private:
		unique_ref<WinHttpSession> session_;

		DISALLOW_COPY_AND_ASSIGN(WinHttpClient);
	};

}

#endif
#endif
