//! Vanilla's post-processing GLSL (`#version 150` with plain `uniform`s, an
//! `in vec4 Position` attribute and `uniform sampler2D`s) turned into naga
//! modules at run time, from whichever resource root is active, the way
//! Horizon's shader packs turn their GLSL into naga (`horizon-shaders`'
//! translator, which handles the compatibility dialect; the post shaders
//! need only the uniform and sampler binding rewrite below).
//!
//! `ShaderManager`/`GlslPreprocessor` read `assets/<ns>/shaders/<path>.{vsh,fsh}` and
//! resolve `#include <ns:path>` to `assets/<ns>/shaders/include/<path>`.
//!
//! The rewrite gives every uniform block a binding of bind group 0, shared by both
//! stages, wraps the plain uniforms of both stages into one synthetic std140
//! block (`_hz_uniforms`, also shared), and splits each `uniform sampler2D
//! NameSampler;` into a texture and a sampler object at the next two bindings,
//! `NameSampler` becoming the combined `sampler2D(texture, sampler)` again
//! where it is used. The vertex stage's `in vec4 Position` attribute becomes
//! the fullscreen triangle's corner from the vertex index (the passes draw
//! with no vertex buffer), and its `main` is wrapped so that clip y is
//! negated: targets keep GL's rows (row 0 is the bottom edge), so `texCoord`
//! reads the texel it would in vanilla.
use naga::ShaderStage;
use naga::front::glsl::{Frontend, Options};
use naga::valid::{Capabilities, ValidationFlags, Validator};
use std::path::Path;

/// One program, ready for wgpu.
pub struct Program {
    pub vertex: naga::Module,
    pub fragment: naga::Module,
    /// Every uniform block either stage declares, with its binding.
    pub blocks: Vec<(String, u32)>,
    /// Every sampler (`NameSampler`): the texture's binding; the sampler object is the next.
    pub samplers: Vec<(String, u32)>,
    /// The plain uniforms of both stages, in first-seen order: the synthetic
    /// block's members.
    pub uniforms: Vec<(String, UniformKind)>,
    /// The synthetic block's binding, when there is one.
    pub plain_binding: Option<u32>,
}

/// A plain uniform's GLSL type, for the synthetic block's std140 layout
/// (`UniformType`'s seven kinds).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UniformKind {
    Int,
    IVec3,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
}

impl UniformKind {
    /// std140 (alignment, size) in bytes.
    pub fn layout(&self) -> (usize, usize) {
        match self {
            UniformKind::Int | UniformKind::Float => (4, 4),
            UniformKind::Vec2 => (8, 8),
            UniformKind::Vec3 | UniformKind::IVec3 => (16, 12),
            UniformKind::Vec4 => (16, 16),
            UniformKind::Mat4 => (16, 64),
        }
    }

    /// `UniformType.getCount`.
    pub fn count(&self) -> usize {
        match self {
            UniformKind::Int | UniformKind::Float => 1,
            UniformKind::Vec2 => 2,
            UniformKind::Vec3 | UniformKind::IVec3 => 3,
            UniformKind::Vec4 => 4,
            UniformKind::Mat4 => 16,
        }
    }

    /// `UniformType.isIntStorage`.
    pub fn is_int(&self) -> bool {
        matches!(self, UniformKind::Int | UniformKind::IVec3)
    }

    fn from_glsl(ty: &str) -> Option<Self> {
        Some(match ty {
            "int" => UniformKind::Int,
            "ivec3" => UniformKind::IVec3,
            "float" => UniformKind::Float,
            "vec2" => UniformKind::Vec2,
            "vec3" => UniformKind::Vec3,
            "vec4" => UniformKind::Vec4,
            "mat4" => UniformKind::Mat4,
            _ => return None,
        })
    }

    /// The GLSL spelling, for the synthetic block.
    fn glsl(&self) -> &'static str {
        match self {
            UniformKind::Int => "int",
            UniformKind::IVec3 => "ivec3",
            UniformKind::Float => "float",
            UniformKind::Vec2 => "vec2",
            UniformKind::Vec3 => "vec3",
            UniformKind::Vec4 => "vec4",
            UniformKind::Mat4 => "mat4",
        }
    }
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The source with its comments blanked (same length, so offsets stay valid).
fn blank_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = b.to_vec();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                if b[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
            for slot in out.iter_mut().take((i + 2).min(b.len())).skip(i) {
                *slot = b' ';
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// `GlslPreprocessor`'s `#include <ns:path>`.
fn read_source(root: &Path, id: &str, ext: &str) -> Result<String, String> {
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let file = root
        .join("assets")
        .join(ns)
        .join("shaders")
        .join(format!("{path}.{ext}"));
    std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))
}

fn include(root: &Path, src: &str, depth: u32) -> Result<String, String> {
    if depth > 8 {
        return Err("#include nested too deep".into());
    }
    let mut out = String::new();
    for line in src.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("#include") {
            let name = rest
                .trim()
                .trim_start_matches(['<', '"'])
                .trim_end_matches(['>', '"'])
                .trim();
            let (ns, path) = name.split_once(':').unwrap_or(("minecraft", name));
            let file = root
                .join("assets")
                .join(ns)
                .join("shaders")
                .join("include")
                .join(path);
            let text = std::fs::read_to_string(&file)
                .map_err(|e| format!("#include <{name}>: {}: {e}", file.display()))?;
            out.push_str(&include(root, &text, depth + 1)?);
        } else if t.starts_with("#version") || t.starts_with("#extension") {
            // Re-added below, for the translator.
            out.push('\n');
            continue;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    Ok(out)
}

/// One `uniform` declaration of a source.
enum Decl {
    /// `uniform sampler2D Name;`: its span (including a `layout(...)` before
    /// it), the sampler type and the name.
    Sampler {
        start: usize,
        end: usize,
        ty: String,
        name: String,
    },
    /// `uniform Name { ... };`: its span up to the name, and the block name.
    Block {
        start: usize,
        end: usize,
        name: String,
    },
    /// `uniform <type> <name>;`: its span, the kind and the name.
    Plain {
        start: usize,
        end: usize,
        kind: UniformKind,
        name: String,
    },
    /// `uniform <type> <name>[...];`: an array, which a chain cannot set.
    Array { name: String },
    /// A `uniform` of a type the synthetic block does not know.
    Unknown { ty: String },
}

/// Every `uniform` declaration of a source.
fn declarations(src: &str) -> Vec<Decl> {
    let plain = blank_comments(src);
    let b = plain.as_bytes();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(found) = plain[at..].find("uniform") {
        let start = at + found;
        at = start + 7;
        if (start > 0 && is_ident(b[start - 1])) || b.get(start + 7).is_some_and(|c| is_ident(*c)) {
            continue;
        }
        let word = |from: usize| -> Option<(usize, usize)> {
            let mut s = from;
            while s < b.len() && b[s].is_ascii_whitespace() {
                s += 1;
            }
            let mut e = s;
            while e < b.len() && is_ident(b[e]) {
                e += 1;
            }
            (e > s).then_some((s, e))
        };
        let Some((ts, te)) = word(start + 7) else {
            continue;
        };
        let ty = plain[ts..te].to_owned();
        // The `layout(...)` right before `uniform`, if any.
        let before = plain[..start].trim_end();
        let decl_start = before
            .ends_with(')')
            .then(|| before.rfind("layout"))
            .flatten()
            .filter(|l| {
                before[*l + 6..].trim_start().starts_with('(') && !before[*l..].contains(';')
            })
            .unwrap_or(start);
        if ty.starts_with("sampler") {
            let Some((ns, ne)) = word(te) else { continue };
            let end = plain[ne..].find(';').map_or(ne, |p| ne + p + 1);
            out.push(Decl::Sampler {
                start: decl_start,
                end,
                ty,
                name: plain[ns..ne].to_owned(),
            });
        } else {
            let mut s = te;
            while s < b.len() && b[s].is_ascii_whitespace() {
                s += 1;
            }
            if b.get(s) == Some(&b'{') {
                out.push(Decl::Block {
                    start: decl_start,
                    end: te,
                    name: ty,
                });
            } else if let Some(kind) = UniformKind::from_glsl(&ty) {
                let Some((ns, ne)) = word(te) else { continue };
                let name = plain[ns..ne].to_owned();
                let mut e = ne;
                while e < b.len() && b[e].is_ascii_whitespace() {
                    e += 1;
                }
                if b.get(e) == Some(&b'[') {
                    out.push(Decl::Array { name });
                    continue;
                }
                let end = plain[ne..].find(';').map_or(ne, |p| ne + p + 1);
                out.push(Decl::Plain {
                    start: decl_start,
                    end,
                    kind,
                    name,
                });
            } else {
                out.push(Decl::Unknown { ty });
            }
        }
    }
    out
}

/// The source with each declaration given its binding from the shared tables.
/// Plain uniforms are gathered into the one synthetic block, at the first
/// one's place.
fn rewrite(
    src: &str,
    blocks: &[(String, u32)],
    samplers: &[(String, u32)],
    plain: &[(String, UniformKind)],
    plain_binding: Option<u32>,
) -> String {
    let mut out = String::new();
    let mut pos = 0;
    let mut emitted = false;
    for decl in declarations(src) {
        match decl {
            Decl::Sampler {
                start,
                end,
                ty,
                name,
            } => {
                out.push_str(&src[pos..start]);
                pos = end;
                let tex = samplers.iter().find(|(n, _)| *n == name).map_or(0, |s| s.1);
                let kind = if ty == "sampler2D" { "texture2D" } else { &ty };
                out.push_str(&format!(
                    "layout(set = 0, binding = {tex}) uniform {kind} {name}_hz_tex; \
                     layout(set = 0, binding = {}) uniform sampler {name}_hz_smp;\n\
                     #define {name} sampler2D({name}_hz_tex, {name}_hz_smp)\n",
                    tex + 1
                ));
            }
            Decl::Block { start, end, name } => {
                out.push_str(&src[pos..start]);
                pos = end;
                let binding = blocks.iter().find(|(n, _)| *n == name).map_or(0, |s| s.1);
                out.push_str(&format!(
                    "layout(std140, set = 0, binding = {binding}) uniform {name}"
                ));
            }
            Decl::Plain { start, end, .. } => {
                out.push_str(&src[pos..start]);
                pos = end;
                if !emitted {
                    emitted = true;
                    if let Some(binding) = plain_binding {
                        out.push_str(&format!(
                            "layout(std140, set = 0, binding = {binding}) uniform _hz_uniforms {{\n"
                        ));
                        for (name, kind) in plain {
                            out.push_str(&format!("    {} {name};\n", kind.glsl()));
                        }
                        out.push_str("};\n");
                    }
                }
            }
            // `translate` rejects these before rewriting.
            Decl::Array { .. } | Decl::Unknown { .. } => {}
        }
    }
    out.push_str(&src[pos..]);
    out
}

/// 1.21.5's vertex shaders read the quad corner from an attribute (`in vec4
/// Position`); the passes draw a fullscreen triangle with no vertex buffer,
/// so the corner comes from the vertex index instead.
fn position_define(src: &str) -> String {
    let plain = blank_comments(src);
    let b = plain.as_bytes();
    let mut at = 0;
    while let Some(found) = plain[at..].find("Position") {
        let p = at + found;
        at = p + 8;
        if (p > 0 && is_ident(b[p - 1])) || b.get(p + 8).is_some_and(|c| is_ident(*c)) {
            continue;
        }
        // Walk back over whitespace to `vec4`, then to `in`.
        let mut s = p;
        while s > 0 && b[s - 1].is_ascii_whitespace() {
            s -= 1;
        }
        if !plain[..s].ends_with("vec4") {
            continue;
        }
        s -= 4;
        while s > 0 && b[s - 1].is_ascii_whitespace() {
            s -= 1;
        }
        if s < 2 || !plain[..s].ends_with("in") || (s > 2 && is_ident(b[s - 3])) {
            continue;
        }
        s -= 2;
        // A `layout(...)` right before `in`, if any.
        let mut start = s;
        let mut t = s;
        while t > 0 && b[t - 1].is_ascii_whitespace() {
            t -= 1;
        }
        if t > 0 && b[t - 1] == b')' {
            if let Some(l) = plain[..t].rfind("layout") {
                if plain[l + 6..].trim_start().starts_with('(') && !plain[l..t].contains(';') {
                    start = l;
                }
            }
        }
        // The `;` after the name.
        let mut e = p + 8;
        while e < b.len() && b[e].is_ascii_whitespace() {
            e += 1;
        }
        if b.get(e) != Some(&b';') {
            continue;
        }
        let mut out = String::with_capacity(src.len() + 130);
        out.push_str(&src[..start]);
        out.push_str("#define Position vec4(vec2(float((gl_VertexIndex << 1) & 2), float(gl_VertexIndex & 2)), 0.0, 1.0)\n");
        out.push_str(&src[e + 1..]);
        return out;
    }
    src.to_owned()
}

/// Replaces the word `main` of `void main(` with `hz_main`.
fn wrap_main(src: &str) -> Result<String, String> {
    let plain = blank_comments(src);
    let mut at = 0;
    while let Some(found) = plain[at..].find("main") {
        let p = at + found;
        at = p + 4;
        let b = plain.as_bytes();
        if (p > 0 && is_ident(b[p - 1])) || b.get(p + 4).is_some_and(|c| is_ident(*c)) {
            continue;
        }
        if plain[..p].trim_end().ends_with("void") && plain[p + 4..].trim_start().starts_with('(') {
            let mut out = String::with_capacity(src.len() + 80);
            out.push_str(&src[..p]);
            out.push_str("hz_main");
            out.push_str(&src[p + 4..]);
            out.push_str("\nvoid main() { hz_main(); gl_Position.y = -gl_Position.y; }\n");
            return Ok(out);
        }
    }
    Err("no main".into())
}

fn parse(stage: ShaderStage, text: &str) -> Result<naga::Module, String> {
    let module = Frontend::default()
        .parse(&Options::from(stage), text)
        .map_err(|e| {
            e.errors
                .iter()
                .map(|e| e.kind.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .map_err(|e| format!("{:?}", e.into_inner()))?;
    Ok(module)
}

/// Translates the vertex and fragment shader a pass names (`minecraft:post/invert`).
pub fn translate(root: &Path, vertex: &str, fragment: &str) -> Result<Program, String> {
    let vs = include(root, &read_source(root, vertex, "vsh")?, 0)?;
    let fs = include(root, &read_source(root, fragment, "fsh")?, 0)?;
    // Blocks, samplers and plain uniforms are shared by name across the
    // stages, so both agree on a binding.
    let mut blocks: Vec<(String, u32)> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut plain: Vec<(String, UniformKind)> = Vec::new();
    for src in [&vs, &fs] {
        for decl in declarations(src) {
            match decl {
                Decl::Sampler { name, .. } => {
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                Decl::Block { name, .. } => {
                    if !blocks.iter().any(|(n, _)| *n == name) {
                        blocks.push((name, blocks.len() as u32));
                    }
                }
                Decl::Plain { kind, name, .. } => match plain.iter().find(|(n, _)| *n == name) {
                    Some((_, old)) if *old != kind => {
                        return Err(format!("uniform {name} has two types"));
                    }
                    Some(_) => {}
                    None => plain.push((name, kind)),
                },
                Decl::Array { name } => {
                    return Err(format!("uniform {name} is an array"));
                }
                Decl::Unknown { ty } => {
                    return Err(format!("unsupported uniform type {ty}"));
                }
            }
        }
    }
    let plain_binding = (!plain.is_empty()).then(|| blocks.len() as u32);
    let first = blocks.len() as u32 + u32::from(plain_binding.is_some());
    let samplers: Vec<(String, u32)> = names.into_iter().zip((first..).step_by(2)).collect();
    let vs = wrap_main(&position_define(&rewrite(&vs, &blocks, &samplers, &plain, plain_binding)))?;
    let fs = rewrite(&fs, &blocks, &samplers, &plain, plain_binding);
    let vertex_module = parse(ShaderStage::Vertex, &format!("#version 450\n{vs}"))
        .map_err(|e| format!("{vertex}: {e}"))?;
    let fragment_module = parse(ShaderStage::Fragment, &format!("#version 450\n{fs}"))
        .map_err(|e| format!("{fragment}: {e}"))?;
    Ok(Program {
        vertex: vertex_module,
        fragment: fragment_module,
        blocks,
        samplers,
        uniforms: plain,
        plain_binding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> Option<std::path::PathBuf> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)?
            .join(".local/horizon-client/1.21.5");
        dir.join("assets/minecraft/shaders/post/blit.fsh")
            .exists()
            .then_some(dir)
    }

    #[test]
    fn vanilla_post_shaders_translate() {
        let Some(root) = root() else { return };
        let vs = [
            "minecraft:post/screenquad",
            "minecraft:post/blit",
            "minecraft:post/blur",
            "minecraft:post/sobel",
            "minecraft:post/rotscale",
            "minecraft:post/invert",
        ];
        let fs = [
            "bits",
            "blit",
            "box_blur",
            "color_convolve",
            "entity_outline_box_blur",
            "entity_sobel",
            "invert",
            "spiderclip",
        ];
        for f in fs {
            for v in vs {
                let name = format!("minecraft:post/{f}");
                let p = translate(&root, v, &name);
                // rotscale's outputs only match spiderclip, but each stage translates alone.
                if let Err(e) = &p {
                    panic!("{v} + {name}: {e}");
                }
            }
        }
    }

    #[test]
    fn sampler_and_block_bindings() {
        let src = "layout(std140) uniform A {\n float x;\n};\nuniform sampler2D InSampler; // c\nvoid main(){}\n";
        let d = declarations(src);
        assert_eq!(d.len(), 2);
        let out = rewrite(src, &[("A".into(), 0)], &[("InSampler".into(), 1)], &[], None);
        assert!(out.contains("layout(std140, set = 0, binding = 0) uniform A {"));
        assert!(out.contains("binding = 1) uniform texture2D InSampler_hz_tex;"));
        assert!(out.contains("binding = 2) uniform sampler InSampler_hz_smp;"));
        assert!(wrap_main("void main() {}").unwrap().contains("hz_main"));
    }

    #[test]
    fn plain_uniforms_share_one_synthetic_block() {
        let src = "uniform float Radius;\nuniform vec2 InSize;\nuniform sampler2D InSampler;\nvoid main(){}\n";
        let d = declarations(src);
        assert_eq!(d.len(), 3);
        let plain = [
            ("Radius".to_owned(), UniformKind::Float),
            ("InSize".to_owned(), UniformKind::Vec2),
        ];
        let out = rewrite(
            src,
            &[],
            &[("InSampler".into(), 1)],
            &plain,
            Some(0),
        );
        // The bare declarations are gone, replaced by the one block.
        assert!(!out.contains("uniform float Radius;"));
        assert!(!out.contains("uniform vec2 InSize;"));
        assert!(out.contains("layout(std140, set = 0, binding = 0) uniform _hz_uniforms {"));
        assert!(out.contains("    float Radius;\n"));
        assert!(out.contains("    vec2 InSize;\n"));
        // A uniform array, and an unknown type, are reported, not rewritten.
        assert!(matches!(
            declarations("uniform vec4 xs[6];\n").into_iter().next(),
            Some(Decl::Array { .. })
        ));
        assert!(matches!(
            declarations("uniform mat3 m;\n").into_iter().next(),
            Some(Decl::Unknown { .. })
        ));
    }

    #[test]
    fn the_position_attribute_becomes_the_triangle_corner() {
        let src = "in vec4 Position;\nvoid main(){ gl_Position = vec4(Position.xy, 0.0, 1.0); }\n";
        let out = position_define(src);
        assert!(!out.contains("in vec4 Position;"));
        assert!(out.contains("#define Position vec4(vec2(float((gl_VertexIndex << 1) & 2), float(gl_VertexIndex & 2)), 0.0, 1.0)"));
        // A `Position` that is not the attribute is left alone.
        let src = "vec4 Position;\nvoid main(){}\n";
        assert_eq!(position_define(src), src);
    }
}
