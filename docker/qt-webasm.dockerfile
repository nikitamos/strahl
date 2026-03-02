FROM emscripten/emsdk:5.0.2 as builder

ARG QT_BRANCH=v6.10.2

# No Qt WebEngine or Qt pdf
RUN apt update && apt install -y cmake ninja-build python3 git build-essential
RUN git clone --branch $QT_BRANCH https://code.qt.io/qt/qt5.git /qt-src
                    
RUN mkdir /build
RUN mkdir /host-build

WORKDIR /qt-src/
RUN ./init-repository --module-subset=essential

WORKDIR /build
RUN apt update && apt install -y libjpeg-turbo8-dev
RUN /qt-src/configure -developer-build -no-opengl -no-warnings-are-errors -nomake tests -prefix /qt-bin
RUN cmake --build . --target host_tools --parallel
RUN cmake --install .


# Minimal host build
# RUN /qt-src/configure -prefix /qt-bin
# RUN cmake --build . --parallel

# WORKDIR /qt-src
# RUN ./init-repository                         \
#     --module-subset=qtbase,qtsvg,qt5compat,qtimageformats,qtwebsockets,qtcharts,qtgraphs
