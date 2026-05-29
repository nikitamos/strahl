
#ifdef GL_ES
precision mediump float;
#endif

uniform float u_time;
uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_segments;
uniform float u_complexity;
uniform float u_colorShift;

void main() {
    vec3 color = vec3(1.0, 1.0, 1.0);
    gl_FragColor = vec4(color, 1.0);
}