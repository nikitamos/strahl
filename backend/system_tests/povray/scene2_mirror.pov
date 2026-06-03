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
}

plane { y, -1 pigment { color rgb <0.8, 0.8, 0.8> } finish { diffuse 1 ambient 0 } }

#declare Mat_White = texture { pigment { color rgb <0.8, 0.8, 0.8> } finish { diffuse 1 ambient 0 } }
#declare Mat_Mirror = texture { 
  pigment { color rgb <0.95, 0.95, 0.95> } 
  finish { diffuse 0.05 ambient 0 reflection { 0.95, 0.95 } } 
}

sphere { <-2, 0, 0>, 1 texture { Mat_White } }
sphere { <2, 0, 0>, 1 texture { Mat_Mirror } }