// GPU instanced renderer for CAD geometry.
//
// Two instanced pipelines share one static unit-quad VBO:
//   * circles — unit quad sized to the radius, SDF ring in the fragment shader.
//   * lines   — unit quad oriented + extruded along a segment, SDF line in the
//               fragment shader (per-instance half-width drives the stroke).
// Curves (arc/ellipse/spline/polyline) are tessellated to line segments on the
// CPU and fed to the line pipeline; the render loop batches everything into two
// draw calls (one per pipeline).
//
// Coordinates are CAMERA-RELATIVE: the CPU emits (world + world_offset) as f32
// (computed in f64 first), and `view_matrix` is built with offset 0. That keeps
// instance magnitudes small near the viewport so f32 stays precise even for
// drawings far from the origin.
//
// This file is touched only from inside the egui PaintCallback closure (GL
// thread). All glow IDs are POD integer handles → Send/Sync.

use std::mem::size_of;

use eframe::glow;
use eframe::glow::HasContext;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CircleInstance {
    pub x:     f32,
    pub y:     f32,
    pub r:     f32,
    pub color: u32,    // packed RGBA, byte order R high … A low
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LineInstance {
    pub ax:     f32,
    pub ay:     f32,
    pub bx:     f32,
    pub by:     f32,
    pub half_w: f32,   // world-space HALF stroke width (screen-min applied CPU-side)
    pub color:  u32,
}

/// One shader program + its instance buffer + VAO (attributes wired).
struct GpuPipeline {
    program:      glow::Program,
    vao:          glow::VertexArray,
    instance_vbo: glow::Buffer,
    u_view:       Option<glow::UniformLocation>,
}

pub struct GpuShapeRenderer {
    circle:   Option<GpuPipeline>,
    line:     Option<GpuPipeline>,
    quad_vbo: Option<glow::Buffer>,
}

// Safety: see module note — glow handles are integer ids; only *use* happens on
// the GL thread (inside the PaintCallback).
unsafe impl Send for GpuShapeRenderer {}
unsafe impl Sync for GpuShapeRenderer {}

impl Default for GpuShapeRenderer {
    fn default() -> Self {
        Self { circle: None, line: None, quad_vbo: None }
    }
}

// ---- shaders ---------------------------------------------------------------

const CIRCLE_VS: &str = r#"
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

// CAD-style ring (outline) ~1 px wide centered on d == 1.0.
const CIRCLE_FS: &str = r#"
    #version 330 core
    in vec2       v_local;
    flat in vec4  v_color;
    out vec4 frag;
    void main() {
        float d  = length(v_local);
        float aa = fwidth(d);
        float thickness = max(1.0 * aa, 0.0012);
        float half_w    = thickness * 0.5;
        if (d > 1.0 + half_w + aa) discard;
        if (d < 1.0 - half_w - aa) discard;
        float dist = abs(d - 1.0);
        float a    = 1.0 - smoothstep(half_w - aa, half_w + aa, dist);
        if (a < 0.005) discard;
        frag = vec4(v_color.rgb, v_color.a * a);
    }
"#;

// Line: the unit quad is oriented along the segment and extruded ±(half_w*PAD)
// perpendicular so the SDF anti-aliasing band has room. Width comes from the
// per-instance half_w (fix: NOT re-derived from fwidth), so lineweights work.
const LINE_VS: &str = r#"
    #version 330 core
    layout(location=0) in vec2  a_quad;      // unit quad corners [-1,1]
    layout(location=1) in vec4  a_line_ab;   // ax,ay,bx,by (camera-relative)
    layout(location=2) in float a_half_w;    // world-space half stroke width
    layout(location=3) in uint  a_color;
    uniform mat4 u_view;
    flat out vec4  v_color;
    flat out vec2  v_a;
    flat out vec2  v_b;
    flat out float v_half_w;
    out vec2       v_pos;
    void main() {
        vec2 a = a_line_ab.xy;
        vec2 b = a_line_ab.zw;
        vec2 dir = b - a;
        float len = length(dir);
        vec2 d = (len > 1e-9) ? dir / len : vec2(1.0, 0.0);
        vec2 n = vec2(-d.y, d.x);
        float PAD = 2.5;                       // AA head-room around the stroke
        float ext = a_half_w * PAD;
        float u = (a_quad.x * 0.5 + 0.5) * len;
        float v = a_quad.y * ext;
        vec2 world = a + d * u + n * v;
        gl_Position = u_view * vec4(world, 0.0, 1.0);
        v_pos = world; v_a = a; v_b = b; v_half_w = a_half_w;
        v_color = vec4(
            float((a_color >> 24) & 0xFFu) / 255.0,
            float((a_color >> 16) & 0xFFu) / 255.0,
            float((a_color >>  8) & 0xFFu) / 255.0,
            float( a_color        & 0xFFu) / 255.0
        );
    }
"#;

const LINE_FS: &str = r#"
    #version 330 core
    flat in vec4  v_color;
    flat in vec2  v_a;
    flat in vec2  v_b;
    flat in float v_half_w;
    in vec2       v_pos;
    out vec4 frag;
    float sd_line(vec2 p, vec2 a, vec2 b) {
        vec2 pa = p - a, ba = b - a;
        float h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-9), 0.0, 1.0);
        return length(pa - ba * h);
    }
    void main() {
        float dist = sd_line(v_pos, v_a, v_b);
        float aa = fwidth(dist);
        float half_w = max(v_half_w, 0.5 * aa);   // never thinner than ~1px on screen
        if (dist > half_w + aa) discard;
        float alpha = 1.0 - smoothstep(half_w - aa, half_w + aa, dist);
        if (alpha < 0.004) discard;
        frag = vec4(v_color.rgb, v_color.a * alpha);
    }
"#;

impl GpuShapeRenderer {
    /// Compile programs + create buffers, idempotently.
    pub fn ensure_init(&mut self, gl: &glow::Context) {
        if self.quad_vbo.is_some() { return; }
        unsafe {
            let quad: [f32; 12] = [
                -1.0, -1.0,  1.0, -1.0,  1.0,  1.0,
                -1.0, -1.0,  1.0,  1.0, -1.0,  1.0,
            ];
            let qvbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(qvbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes(&quad), glow::STATIC_DRAW);
            self.quad_vbo = Some(qvbo);

            self.circle = Some(Self::build_circle(gl, qvbo));
            self.line   = Some(Self::build_line(gl, qvbo));

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
        }
    }

    unsafe fn compile(gl: &glow::Context, vs_src: &str, fs_src: &str) -> glow::Program {
        let program = gl.create_program().expect("create_program");
        let compile = |src: &str, kind: u32| -> glow::Shader {
            let s = gl.create_shader(kind).expect("create_shader");
            gl.shader_source(s, src);
            gl.compile_shader(s);
            if !gl.get_shader_compile_status(s) {
                panic!("GPU shader compile failed:\n{}", gl.get_shader_info_log(s));
            }
            s
        };
        let vs = compile(vs_src, glow::VERTEX_SHADER);
        let fs = compile(fs_src, glow::FRAGMENT_SHADER);
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("GPU program link failed:\n{}", gl.get_program_info_log(program));
        }
        gl.delete_shader(vs);
        gl.delete_shader(fs);
        program
    }

    unsafe fn build_circle(gl: &glow::Context, quad_vbo: glow::Buffer) -> GpuPipeline {
        let program = Self::compile(gl, CIRCLE_VS, CIRCLE_FS);
        let u_view = gl.get_uniform_location(program, "u_view");
        let ivbo = gl.create_buffer().unwrap();
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(ivbo));
        let stride = size_of::<CircleInstance>() as i32;
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 0);
        gl.vertex_attrib_divisor(1, 1);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_i32(2, 1, glow::UNSIGNED_INT, stride, 12);
        gl.vertex_attrib_divisor(2, 1);
        GpuPipeline { program, vao, instance_vbo: ivbo, u_view }
    }

    unsafe fn build_line(gl: &glow::Context, quad_vbo: glow::Buffer) -> GpuPipeline {
        let program = Self::compile(gl, LINE_VS, LINE_FS);
        let u_view = gl.get_uniform_location(program, "u_view");
        let ivbo = gl.create_buffer().unwrap();
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(ivbo));
        let stride = size_of::<LineInstance>() as i32;   // 24
        gl.enable_vertex_attrib_array(1);                // ax,ay,bx,by
        gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, stride, 0);
        gl.vertex_attrib_divisor(1, 1);
        gl.enable_vertex_attrib_array(2);                // half_w
        gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, stride, 16);
        gl.vertex_attrib_divisor(2, 1);
        gl.enable_vertex_attrib_array(3);                // color
        gl.vertex_attrib_pointer_i32(3, 1, glow::UNSIGNED_INT, stride, 20);
        gl.vertex_attrib_divisor(3, 1);
        GpuPipeline { program, vao, instance_vbo: ivbo, u_view }
    }

    /// Upload both instance buffers and draw (one call per non-empty pipeline).
    pub fn render(
        &mut self,
        gl: &glow::Context,
        circles: &[CircleInstance],
        lines: &[LineInstance],
        view: &[f32; 16],
    ) {
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        Self::draw(gl, &self.circle, bytes(circles), circles.len(), view);
        Self::draw(gl, &self.line,   bytes(lines),   lines.len(),   view);
        unsafe { gl.use_program(None); }
    }

    fn draw(
        gl: &glow::Context,
        pipe: &Option<GpuPipeline>,
        data: &[u8],
        count: usize,
        view: &[f32; 16],
    ) {
        if count == 0 { return; }
        let p = match pipe { Some(p) => p, None => return };
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(p.instance_vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::DYNAMIC_DRAW);
            gl.use_program(Some(p.program));
            if let Some(loc) = &p.u_view {
                gl.uniform_matrix_4_f32_slice(Some(loc), false, view);
            }
            gl.bind_vertex_array(Some(p.vao));
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, count as i32);
            gl.bind_vertex_array(None);
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
/// callback rect's centre is at clip (0, 0) and the rect's half-extents are ±1.
/// Matches CPU `w2s` semantics. With camera-relative instances, pass ox = oy = 0
/// (the world_offset is already folded into the instance coordinates).
pub fn view_matrix(rect_w: f32, rect_h: f32, scale: f32, ox: f32, oy: f32) -> [f32; 16] {
    let sx = 2.0 * scale / rect_w;
    let sy = 2.0 * scale / rect_h;
    [
        sx,       0.0,      0.0, 0.0,
        0.0,      sy,       0.0, 0.0,
        0.0,      0.0,      1.0, 0.0,
        sx * ox,  sy * oy,  0.0, 1.0,
    ]
}
