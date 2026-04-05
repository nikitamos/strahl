#pragma once
#include <concepts>
#include <cstddef>
#include <functional>

namespace strahl::cpu {
template <typename S>
concept space = requires(S s) { typename S::Point; };

template<typename P>
struct Sample {
  P point;
  float prob;
};

class Space {};
template <space S>
class Domain {
  template<typename Sampler>
  Sample<typename S::Point> samplePoint(Sampler s) { return {}; }
  float measure();
};

class RaySpace {};

// Who is a sampler?
// Who is responsible for point sampling?

template <space S>
class MonteCarloIntegrator {
  template<typename Sampler>
  float integrate(Domain<S> dom, std::function<float(typename S::Point)> f, size_t samples, Sampler sampler) {
    float res = 0;
    for (size_t s = 0; s < samples; ++s) {
      auto [point, prob] = dom.samplePoint(sampler);
      res += f(point) / prob;
    }
    return res;
  }
};
}  // namespace strahl::cpu
