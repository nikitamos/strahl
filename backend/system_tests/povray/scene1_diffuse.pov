#include "colors.inc"

camera {
  location <0, 2, -8>
  look_at <0, 0, 0>
  angle 40
}

light_source {
  <0, 5, 0>
  color rgb <10, 10, 10>
  area_light <4, 0, 0>, <0, 0, 4>, 5, 5
  adaptive 2
  jitter
}

plane {
  y, -1
  pigment { color rgb <0.8, 0.8, 0.8> }
  finish { diffuse 1 ambient 0 }
}

#declare Mat_Red = texture { pigment { color rgb <0.8, 0.1, 0.1> } finish { diffuse 1 ambient 0 } }
#declare Mat_Green = texture { pigment { color rgb <0.1, 0.8, 0.1> } finish { diffuse 1 ambient 0 } }
#declare Mat_Blue = texture { pigment { color rgb <0.1, 0.1, 0.8> } finish { diffuse 1 ambient 0 } }

sphere { <-2.5, 0, 0>, 1 texture { Mat_Red } }
sphere { <0, 0, 0>, 1 texture { Mat_Green } }
sphere { <2.5, 0, 0>, 1 texture { Mat_Blue } }