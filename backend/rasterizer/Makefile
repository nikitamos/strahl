
SLANG=/opt/shader-slang-bin/bin/slangc

shaders/raster-pipeline.wgsl: shaders/raster-pipeline.slang
	$(SLANG) shaders/raster-pipeline.slang -o shaders/raster-pipeline.wgsl
	sed -i "/var<uniform> body_0 : Body_std430_0;/c\var<immediate> body_0 : Body_std430_0;" shaders/raster-pipeline.wgsl

