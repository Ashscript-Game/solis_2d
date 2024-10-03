use bevy::{
    prelude::*,
    render::{
        render_resource::{ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
        Extract,
    },
    window::{PrimaryWindow, WindowResized},
};

use crate::{constant, prelude::GiConfig, targets::RenderTargets};

#[derive(Resource, Clone, Debug, ShaderType, Default)]
pub struct ComputedSize {
    pub native: IVec2,
    pub scaled: IVec2,
    pub cascade_size: IVec2,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ComputedSizeBuffer(UniformBuffer<ComputedSize>);

#[rustfmt::skip]
impl ComputedSize {
    pub fn from_window(
        window: &Window,
        scale: i32,
        cascade_count : u32,
    ) -> Self {
        let width = window.physical_width();
        let height = window.physical_height();
        let mut size = IVec2::new(width as i32, height as i32);

        if size.x%2 != 0 {
            size.x += 1;
        }

        if size.y%2 != 0 {
            size.y += 1;
        }

        let mut downscaled_size = size/scale;//+ (scale - size%scale);
        if size.x%scale  > 0 {
            downscaled_size.x += scale - size.x%scale;
        }

        if size.y%scale  > 0 {
            downscaled_size.y += scale - size.y%scale;
        }

        let probe_size =  downscaled_size * IVec2::new(4, 1);

        Self {
            native: size,
            scaled: downscaled_size,
            cascade_size: probe_size,
        }
    }
}

pub fn extract_size(mut buffer: ResMut<ComputedSizeBuffer>, size: Extract<Res<ComputedSize>>) {
    let buffer = buffer.get_mut();
    buffer.native = size.native;
    buffer.scaled = size.scaled;
    buffer.cascade_size = size.cascade_size;
}

pub fn prepare_bindgroup(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut buffer: ResMut<ComputedSizeBuffer>,
) {
    buffer.write_buffer(&render_device, &render_queue);
}

pub fn on_startup(
    mut cmd: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    config: Res<GiConfig>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let computed_size =
        ComputedSize::from_window(&window, config.scale_factor, config.cascade_count);
    let targets = RenderTargets::from_size(&computed_size, &config, &mut images);
    cmd.insert_resource(targets);
    cmd.insert_resource(computed_size);
}

pub fn on_win_resize(mut events: EventReader<WindowResized>, mut cmd: Commands) {
    let Some(event) = events.read().next() else {
        return;
    };

    cmd.trigger(ResizeEvent);
}

#[derive(Event)]
pub struct ResizeEvent;

pub fn resize(
    trigger: Trigger<ResizeEvent>,
    window: Query<&Window, With<PrimaryWindow>>,
    config: Res<GiConfig>,
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let computed_size =
        ComputedSize::from_window(&window, config.scale_factor, config.cascade_count);
    let targets = RenderTargets::from_size(&computed_size, &config, &mut images);
    cmd.insert_resource(computed_size);
    cmd.insert_resource(targets);
}
