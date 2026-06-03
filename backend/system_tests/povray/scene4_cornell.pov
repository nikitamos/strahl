camera {
  location <0, 0, -7>
  look_at <0, 0, 0>
  angle 45
}

// Light source
polygon {
  5, <-1, 2.99, -1>, <1, 2.99, -1>, <1, 2.99, 1>, <-1, 2.99, 1>, <-1, 2.99, -1>
  pigment { color rgb <15, 15, 15> }
  finish { ambient 1 diffuse 0 }
}

// Walls
plane { -x, -3 pigment { color rgb <0.7, 0.1, 0.1> } finish { diffuse 1 ambient 0 } }
plane { x, -3 pigment { color rgb <0.1, 0.7, 0.1> } finish { diffuse 1 ambient 0 } }
plane { -z, -3 pigment { color rgb <0.8, 0.8, 0.8> } finish { diffuse 1 ambient 0 } }
plane { y, -3 pigment { color rgb <0.8, 0.8, 0.8> } finish { diffuse 1 ambient 0 } }
plane { -y, -3 pigment { color rgb <0.8, 0.8, 0.8> } finish { diffuse 1 ambient 0 } }

#declare Mat_Mirror = texture { pigment { color rgb <0.95, 0.95, 0.95> } finish { diffuse 0.05 ambient 0 reflection 0.95 } }
#declare Mat_Glass = texture { 
  pigment { color rgbf <0.95, 0.95, 1.0, 1.0> } 
  finish { ambient 0 diffuse 0.05 refraction 1 reflection { 0.0, 1.0 fresnel on } } 
  interior { ior 1.5 }
}

sphere { <-1.2, -1.5, 0.5>, 1 texture { Mat_Mirror } }
sphere { <1.2, -1.5, -0.5>, 1 texture { Mat_Glass } }