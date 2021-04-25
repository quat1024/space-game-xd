#version 450 core

layout(location=0) in vec2 a_pos;
layout(location=1) in vec3 a_color;

layout(location=0) out vec3 v_color;

layout(set=0,binding=0)
uniform Uniforms {
	mat4 pixel_to_ndc;
};

void main() {
	v_color = a_color;
	
	gl_Position = pixel_to_ndc * vec4(a_pos, 0.0, 1.0); //z = 0 for now (no projection matrix set up)
}