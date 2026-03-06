#include "CImg.h"
#include "raymarcher.hpp"
#include "rm-scene.hpp"
#include <glm/glm.hpp>
#include <iostream>

// using namespace strahl;
using namespace strahl::cpu_raymarcher;

int main(int argc, char **argv) {
  std::cout << "Strahl CLI\n"
            << "==========\n";
  auto back = CpuRaymarcherBackend();
  auto *left = new Plane({0, 3, 0}, {1, -1, 0}, {{1.0, 0.0, 0.0}}),
       *right = new Plane({0, 3, 0}, {-1, -1, 0}, {{0.0, 1.0, 0.0}});
  auto *sphere = new Sphere({0, 1.5, 0}, 0.5, {{0.0, 0.0, 1.0}});
  CompositionNode composite{left, right, sphere};
  back.SetRoot(&composite);
  back.GetOptions().resolution = {48, 36};
  auto res = back.Render();

  cimg_library::CImg<float> img(
      reinterpret_cast<const float *>(res.image.data()), res.resolution.x,
      res.resolution.y, 1, 3);
  img *= 255.0;

  img.save_png("render.png");
}
