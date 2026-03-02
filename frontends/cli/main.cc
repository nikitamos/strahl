#include <iostream>
#include "CImg.h"
#include "raymarcher.hpp"

using namespace strahl;

int main(int argc, char** argv) {
  std::cout << "Strahl CLI\n"
            << "==========\n";
  auto back = cpu_raymarcher::CpuRaymarcherBackend();
  
}