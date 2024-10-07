#define_import_path lommix_light::common

const PI:f32  = 3.14159;
const TAU:f32 = PI * 2.;
const EPSILON: f32 = 4.88e-04;

struct Probe {
    cascade_count: u32,
    cascade_index: u32,
    cascade_interval: f32,
    base: u32,
}

struct GiConfig{
	native: vec2<u32>,
	scaled: vec2<u32>,
	scale: f32,
	cascade_count: u32,
	flags: u32,
}


fn debug_sdf(cfg: GiConfig) -> f32{
	return select(0.,1., (( cfg.flags & 0x1 )!= 0));
}

fn debug_voronoi(cfg: GiConfig) -> f32{
	return select(0.,1., (( cfg.flags >> 1 & 0x1 )!= 0));
}

fn debug_merge0(cfg: GiConfig) -> f32{
	return select(0.,1., ( cfg.flags >> 2 & 0x1 )!= 0);
}

fn debug_merge1(cfg: GiConfig) -> f32{
	return select(0.,1., ( cfg.flags >> 3 & 0x1 )!= 0);
}

fn random(st : vec2<f32>) -> f32 {
   return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
}

