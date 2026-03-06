#include "include/ray.hpp"

namespace strahl::cpu_raymarcher {
void Ray::Advance(float distance) { cur_pos_ += direction_ * distance; }
} // namespace strahl::cpu_raymarcher
