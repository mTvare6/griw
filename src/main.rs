use {
    crate::{
        camera::Camera,
        math::Vec3
    }, anyhow::{Context, Result}, winit::{
        event::{DeviceEvent, Event, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    }
};

use std::time::Instant;

mod render;
mod math;
mod camera;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 225;

#[pollster::main]
async fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window_size = winit::dpi::PhysicalSize::new(WIDTH, HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_resizable(false)
        .with_title("griw".to_string())
        .build(&event_loop)?;

    let (device, queue, surface) = connect_to_gpu(&window).await?;
    let mut renderer = render::PathTracer::new(device, queue, WIDTH, HEIGHT);
    let mut camera = Camera::new(
        Vec3::new(0., 0.5, 1.),
        Vec3::new(0., 0.5, -1.),
        Vec3::new(0., 1., 0.),
    );

    let mut now = Instant::now();

    event_loop.run(|event, control_handle| {
        control_handle.set_control_flow(ControlFlow::Poll);
        use winit::keyboard::PhysicalKey::Code;
        use winit::keyboard::KeyCode::*;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_handle.exit(),
                WindowEvent::RedrawRequested => {
                    let frame: wgpu::SurfaceTexture = surface
                        .get_current_texture()
                        .expect("failed to get current texture");

                    let dt = now.elapsed().as_secs_f64();
                    now = Instant::now();
                    print!("\rFPS: {:.0}  ", dt.recip());
                    let target = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    renderer.render_frame(&target, &camera);

                    frame.present();
                    window.request_redraw();
                }
                WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                    Code(KeyZ) => {camera.zoom(0.1);renderer.reset_samples()},
                    Code(KeyX) => {camera.zoom(-0.1);renderer.reset_samples()},
                    Code(KeyW) => {camera.move_along_w(0.1);renderer.reset_samples()},
                    Code(KeyS) => {camera.move_along_w(-0.1);renderer.reset_samples()},
                    Code(KeyA) => {camera.move_along_u(0.1);renderer.reset_samples()},
                    Code(KeyD) => {camera.move_along_u(-0.1);renderer.reset_samples()},
                    _ => (),
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseWheel { delta } => {
                    let delta = match delta {
                        MouseScrollDelta::PixelDelta(delta) => 0.001 * delta.y as f32,
                        MouseScrollDelta::LineDelta(_, y) => y * 0.001,
                    };
                    camera.zoom(delta);
                    renderer.reset_samples();
                },
                DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                    let dx = dx as f32 *  0.1;
                    let dy = dy as f32 * -0.1;
                    camera.rotate(dx, dy);
                    renderer.reset_samples()
                }
                _ => (),
            }
            _ => (),
        }
    })?;
    Ok(())
}

async fn connect_to_gpu(window: &Window) -> Result<(wgpu::Device, wgpu::Queue, wgpu::Surface)> {
    use wgpu::TextureFormat::{Bgra8Unorm, Rgba8Unorm};

    // Create an "instance" of wgpu. This is the entry-point to the API.
    let instance = wgpu::Instance::default();

    // Create a drawable "surface" that is associated with the window.
    let surface = instance.create_surface(window)?;

    // Request a GPU that is compatible with the surface. If the system has multiple GPUs then
    // pick the high performance one.
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .context("failed to find a compatible adapter")?;

    adapter.features()
        .iter()
        . find(|&i| matches!(i, wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES))
        .context("unable to support rwtextures")?;


    // Connect to the GPU. "device" represents the connection to the GPU and allows us to create
    // resources like buffers, textures, and pipelines. "queue" represents the command queue that
    // we use to submit commands to the GPU.
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor{
            label: Some("making device"),
            required_limits: wgpu::Limits::default(),
            required_features: wgpu::Features::default() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        }, None)
        .await
        .context("failed to connect to the GPU")?;


    // Configure the texture memory backs the surface. Our renderer will draw to a surface texture
    // every frame.
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .into_iter()
        .find(|it| matches!(it, Rgba8Unorm | Bgra8Unorm))
        .context("could not find preferred texture format (Rgba8Unorm or Bgra8Unorm)")?;

    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 1,
    };
    surface.configure(&device, &config);

    Ok((device, queue, surface))
}
