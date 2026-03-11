#include <qguiapplication.h>
#include <qlabel.h>
#include <qsurface.h>
#include <qwindow.h>

#include <QGuiApplication>
#include <iostream>
#include <strahl-vulkan.hpp>

int main(int argc, char **argv) {
  // QGuiApplication app(argc, argv);
  // QWindow w;
  // w.show();
  // auto surfaceType = w.surfaceType();
  // if (surfaceType != QSurface::VulkanSurface) {
  //   std::cerr << "Warn: non-Vulkan window" << std::endl;
  //   w.setSurfaceType(QSurface::VulkanSurface);
  // }
  // std::cout << w.surfaceType() << " " << QSurface::VulkanSurface <<
  // std::endl;
  strahl::vulkan::VulkanBackend b;

  return 0;
}