these are my performance notes

Instance, device, queue, shader modules and shader pipelines have been moved out of the shader_process loop which shaves ~60% off of time spent during frame processing. Bindgroups must stay in the loop to update render targets and other uniforms.

image::ImageReader::open() uses ~0.05µs/pixel (~26ms/frame for 720x720)
DynamicImage::to_rgba8() uses ~0.06µs/pixel (~30ms/frame for 720x720)
These two functions are the culprits for at least 90% of time spent during frame processing.

Time used to open() and convert to_rgba8()
png: ~0.10µs/frame
jpg: ~0.25µs/frame
qoi: ~0.13µs/frame


wgpu::Instance::new() took ~43.5ms
Instance.request_adapter() took ~6.0ms
Adapter.request_device() took ~8.6ms