#include "CImg.h"
#include "raymarcher.hpp"
#include "rm-scene.hpp"
#include <iostream>

// using namespace strahl;
using namespace strahl::cpu_raymarcher;

int main(int argc, char **argv) {
  std::cout << "Strahl CLI\n"
            << "==========\n";
  auto back = CpuRaymarcherBackend();
  auto *left = new Plane({0, 3, 0}, {1, -1, 0}),
       *right = new Plane({0, 3, 0}, {-1, -1, 0});
  auto *sphere = new Sphere({0, 1.5, 0}, 0.5);
  CompositionNode composite{left, right, sphere};
  back.SetRoot(&composite);
  back.GetOptions().resolution = {480, 360};
  back.Render();
}