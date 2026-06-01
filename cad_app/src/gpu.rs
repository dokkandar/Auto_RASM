// GPU instanced renderer for circles (first slice).
//
// One quad in a static VBO, one per-instance buffer of (x, y, r, color_rgba).
// The vertex shader places the quad in world space; the fragment shader
// discards anything outside the unit disk. Single draw call for all N circles.
//
// This file is touched only from inside the egui PaintCallback closure (which
// runs on the GL thread). All glow IDs are u32 underneath and are Send/Sync.

use std::mem::size_of;

use eframe::glow;
use eframe::glow::HasContext;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CircleInstance {
    pub x:     f32,
    pub y:     f32,
    pub r:     f32,
    pub color: u32,    // packed RGBA, big-endian byte order: R high, A low
}

pub struct GpuCircleRenderer {
    program:      Option<glow::Program>,
    vao:          Option<glow::VertexArray>,
    quad_vbo:     Option<glow::Buffer>,
    instance_vbo: Option<glow::Buffer>,
    u_view:       Option<glow::UniformLocation>,
}

// Safety: glow's `Program`/`Buffer`/etc. are POD wrappers around an integer
// resource id. Transferring them across threads doesn't dereference any GL
// state — only *using* them must happen on the GL thread, which is exactly
// where the egui PaintCallback runs.
unsafe impl Send for GpuCircleRenderer {}
unsafe impl Sync for GpuCircleRenderer {}

impl Default for GpuCircleRenderer {
    fn default() -> Self {
        Self {
            program: None, vao: None,
            quad_vbo: None, instance_vbo: None,
            u_view: None,
        }
    }
}

impl GpuCircleRenderer {
    /// Compile the program + create buffers, idempotently.
    pub fn ensure_init(&mut self, gl: &glow::Context) {
        if self.program.is_some() { return; }

        const VS: &str = r#"
            #version 330 core
            layout(location=0) in vec2  a_quad;
            layout(location=1) in vec3  a_circ;     // x, y, r
            layout(location=2) in uint  a_color;
            uniform mat4 u_view;
            out vec2       v_local;
            flat out vec4  v_color;
            void main() {
                v_local = a_quad;
                vec2 world = a_circ.xy + a_quad * a_circ.z;
                gl_Position = u_view * vec4(world, 0.0, 1.0);
                v_color = vec4(
                    float((a_color >> 24) & 0xFFu) / 255.0,
                    float((a_color >> 16) & 0xFFu) / 255.0,
                    float((a_color >>  8) & 0xFFu) / 255.0,
                    float( a_color        & 0xFFu) / 255.0
                );
            }
        "#;

        // Fragment shader: draws a CAD-style ring (outline) ~1 px wide centered
        // on the geometric radius (d = 1.0), matching CPU `circle_stroke` width.
        // `fwidth(d)` gives ≈ 1 / radius_in_px, so multiplying by N produces an
        // N-pixel band in local space.
        const FS: &str = r#"
            #version 330 core
            in vec2       v_local;
            flat in vec4  v_color;
            out vec4 frag;
            void main() {
                float d  = length(v_local);
                float aa = fwidth(d);                       // ≈ 1 / r_px
                // 1.0 px line width — feel free to bump to 1.5 if too thin.
                float thickness = max(1.0 * aa, 0.0012);
                float half_w    = thickness * 0.5;
                // band centered on d == 1.0 (both inside + outside the radius)
                if (d > 1.0 + half_w + aa) discard;
                if (d < 1.0 - half_w - aa) discard;
                float dist = abs(d - 1.0);
                float a    = 1.0 - smoothstep(half_w - aa, half_w + aa, dist);
                if (a < 0.005) discard;
                frag = vec4(v_color.rgb, v_color.a * a);
            }
        "#;

        unsafe {
            let program = gl.create_program().expect("create_program");
            let compile = |src: &str, kind: u32| -> glow::Shader {
                let s = gl.create_shader(kind).expect("create_shader");
                gl.shader_source(s, src);
                gl.compile_shader(s);
                if !gl.get_shader_compile_status(s) {
                    panic!("GPU shader compile failed:\n{}",
                           gl.get_shader_info_log(s));
                }
                s
            };
            let vs = compile(VS, glow::VERTEX_SHADER);
            let fs = compile(FS, glow::FRAGMENT_SHADER);
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("GPU program link failed:\n{}",
                       gl.get_program_info_log(program));
            }
            gl.delete_shader(vs);
            gl.delete_shader(fs);

            let u_view = gl.get_uniform_location(program, "u_view");

            // Static unit quad (two triangles, vertex pos in [-1, 1])
            let quad: [f32; 12] = [
                -1.0, -1.0,  1.0, -1.0,  1.0,  1.0,
                -1.0, -1.0,  1.0,  1.0, -1.0,  1.0,
            ];
            let quad_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytes(&quad),
                glow::STATIC_DRAW,
            );

            let instance_vbo = gl.create_buffer().unwrap();

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            // attrib 0: quad pos (vec2)
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);

            // attribs 1+2: per-instance circle data
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_vbo));
            let stride = size_of::<CircleInstance>() as i32;
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 0);
            gl.vertex_attrib_divisor(1, 1);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_i32(2, 1, glow::UNSIGNED_INT, stride, 12);
            gl.vertex_attrib_divisor(2, 1);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            self.program      = Some(program);
            self.vao          = Some(vao);
            self.quad_vbo     = Some(quad_vbo);
            self.instance_vbo = Some(instance_vbo);
            self.u_view       = u_view;
        }
    }

    /// Upload the instance buffer and issue one draw call.
    pub fn upload_and_render(
        &mut self,
        gl: &glow::Context,
        instances: &[CircleInstance],
        view: &[f32; 16],
    ) {
        if instances.is_empty() { return; }
        let (program, vao, ivbo) =
            match (self.program, self.vao, self.instance_vbo) {
                (Some(p), Some(v), Some(i)) => (p, v, i),
                _ => return,
            };
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(ivbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytes(instances),
                glow::DYNAMIC_DRAW,
            );

            gl.use_program(Some(program));
            if let Some(loc) = &self.u_view {
                gl.uniform_matrix_4_f32_slice(Some(loc), false, view);
            }
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            gl.bind_vertex_array(Some(vao));
            gl.draw_arrays_instanced(
                glow::TRIANGLES, 0, 6, instances.len() as i32);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

/// Reinterpret a `&[T]` of `Copy` POD as bytes, for `glBufferData`.
fn bytes<T: Copy>(slice: &[T]) -> &[u8] {
    let len = std::mem::size_of_val(slice);
    // SAFETY: T is Copy (POD-like) and we never write through this view.
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
}

/// Orthographic-style 4×4 mapping world coords → clip space such that the
/// callback rect's centre is at clip (0, 0) and the rect's half-extents are
/// ±1. Matches the CPU `w2s` semantics: positive `world_offset` pans content
/// right / up in the viewport.
pub fn view_matrix(rect_w: f32, rect_h: f32, scale: f32, ox: f32, oy: f32) -> [f32; 16] {
    let sx = 2.0 * scale / rect_w;
    let sy = 2.0 * scale / rect_h;
    // column-major (OpenGL convention)
    [
        sx,       0.0,      0.0, 0.0,
        0.0,      sy,       0.0, 0.0,
        0.0,      0.0,      1.0, 0.0,
        sx * ox,  sy * oy,  0.0, 1.0,
    ]
}
