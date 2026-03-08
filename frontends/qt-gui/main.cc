#include <QGuiApplication>
#include <iostream>
#include <qguiapplication.h>
#include <qsurface.h>
#include <qwindow.h>
#include <qlabel.h>

int main(int argc, char **argv) {
  QGuiApplication app(argc, argv);
  QWindow w;
  w.show();
  auto surfaceType = w.surfaceType();
  if (surfaceType != QSurface::VulkanSurface) {
    std::cerr << "Warn: non-Vulkan window" << std::endl;
    w.setSurfaceType(QSurface::VulkanSurface);
  }
  std::cout << w.surfaceType();
//   w.
//   QLabel *lbl = new QLabel(&w, "Hello?");
  return QGuiApplication::exec();
}