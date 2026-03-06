#include <boost/gil.hpp>
#include <boost/gil/extension/dynamic_image/any_image.hpp>
#include <boost/gil/extension/io/png.hpp>
#include <boost/gil/image_view.hpp>
#include <boost/mp11.hpp>

#include <glm/fwd.hpp>
#include <iostream>

// #include <Magick++.h>
#include <boost/gil.hpp>
#include <glm/glm.hpp>

#include "raymarcher.hpp"
#include "rm-scene.hpp"

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

  auto data_view = boost::gil::interleaved_view(
      48, 36, (const boost::gil::rgb8_pixel_t *)res.image.data(),
      sizeof(glm::vec3) * 48);
  boost::gil::rgb32f_image_t img(36, 48);
  boost::gil::write_view("render-boost.png", data_view, boost::gil::png_tag{});

  // Magick::Blob blob(res.image.data(),
  //                   res.image.size() *
  //                   sizeof(decltype(res.image)::value_type));
  // Magick::Image img(blob, Magick::Geometry(48, 36), );
  // MagickCore::BlobToImage(MagickCore::ImageInfo{}, const void *, const
  // size_t,
  //                         ExceptionInfo *)
  //     MagickCore

  // img.save_png("render.png");
}
