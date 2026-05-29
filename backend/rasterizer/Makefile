
SLANG=slangc

all: shaders/raster-pipeline.wgsl shaders/blur.wgsl
shaders/raster-pipeline.wgsl: shaders/raster-pipeline.slang
	$(SLANG) $< -o $@
	sed -i "/var<uniform> body_0 : Body_std430_0;/c\var<immediate> body_0 : Body_std430_0;" $@


shaders/blur.wgsl: shaders/blur.slang
	$(SLANG) $^ -o $@
	sed -i "/var<uniform> push_0 : PushConstants_std430_0;/c\var<immediate> push_0 : PushConstants_std430_0;" $@
