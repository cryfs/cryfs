//===----------------------------------------------------------------------===//
// Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
// copy at https://opensource.org/licenses/BSD-3-Clause).
// SPDX-License-Identifier: BSD-3-Clause
//===----------------------------------------------------------------------===//

#include <array>
#include <cstdint>

#include <crypto++/osrng.h> // for random number generation

int main(int argc, char **argv) {
  constexpr size_t c_buffer_size = 16;
  std::array<uint8_t, c_buffer_size> output;
  CryptoPP::AutoSeededRandomPool rng;
  rng.GenerateBlock(output.data(), c_buffer_size);
}