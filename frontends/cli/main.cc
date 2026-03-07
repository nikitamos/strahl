#include <boost/gil.hpp>
#include <boost/gil/algorithm.hpp>
#include <boost/gil/extension/dynamic_image/any_image.hpp>
#include <boost/gil/extension/io/png.hpp>
#include <boost/gil/image.hpp>
#include <boost/gil/image_view.hpp>
#include <boost/gil/typedefs.hpp>
#include <boost/mp11.hpp>

#include <glm/fwd.hpp>
#include <iostream>

// #include <Magick++.h>
#include <boost/gil.hpp>
#include <glm/glm.hpp>
#include <type_traits>

#include "raymarcher.hpp"
#include "rm-scene.hpp"

// using namespace strahl;
using namespace strahl::cpu_raymarcher;
namespace gil = boost::gil;

int main(int argc, char **argv) {
  std::cout << "Strahl CLI\n"
            << "==========\n";
  auto back = CpuRaymarcherBackend();
  auto *left = new Plane({0, 3, 0}, {1, -1, 0}, {{1.0, 0.0, 0.0}}),
       *right = new Plane({0, 3, 0}, {-1, -1, 0}, {{0.0, 1.0, 0.0}}),
       *bottom = new Plane({0, 0, -2.0}, {0, 0, 1},
                           {{1.0, 1.0, 1.0}, 0.8, 0.2, 0.0, 1.0});
  auto *sphere = new Sphere({0, 1.5, 0}, 0.5, {{0.0, 0.0, 1.0}});
  CompositionNode composite{left, right, sphere, bottom};
  back.SetRoot(&composite);
  auto &opts = back.GetOptions();
  opts.resolution = {480, 360};
  opts.bounces = 2;

  auto res = back.Render();

  gil::rgb32f_view_t data_view =
      gil::interleaved_view(opts.resolution.x, opts.resolution.y,
                            (gil::rgb32f_pixel_t *)res.image.data(),
                            sizeof(gil::rgb32f_pixel_t) * opts.resolution.x);
  gil::rgb8_planar_image_t transformed_img(opts.resolution.x,
                                           opts.resolution.y);
  auto transformed_view = gil::view(transformed_img);
  gil::transform_pixels(data_view, transformed_view, [](gil::rgb32f_pixel_t x) {
    return gil::rgb8_pixel_t(x.at(std::integral_constant<int, 0>{}) * 255,
                             x.at(std::integral_constant<int, 1>{}) * 255,
                             x.at(std::integral_constant<int, 2>{}) * 255);
  });
  gil::write_view("render-boost.png", transformed_view, boost::gil::png_tag{});
}
