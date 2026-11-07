//! Growth of the double-buffered quad and glyph instance buffers.

use super::Renderer;

impl Renderer {
    /// Grow the current quad buffer if needed_bytes exceeds capacity.
    pub(super) fn ensure_quad_capacity(&mut self, needed_bytes: u64, buf_idx: usize) {
        if needed_bytes > self.quad_capacities[buf_idx] {
            let new_cap = (self.quad_capacities[buf_idx] as f64 * 1.5) as u64;
            let new_cap = new_cap.max(needed_bytes);
            self.quad_buffers[buf_idx] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("acme quad buffer"),
                size: new_cap,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.quad_capacities[buf_idx] = new_cap;
            self.stats.buffer_grows += 1;
        }
    }

    /// Grow the current glyph buffer if needed_bytes exceeds capacity.
    pub(super) fn ensure_glyph_capacity(&mut self, needed_bytes: u64, buf_idx: usize) {
        if needed_bytes > self.glyph_capacities[buf_idx] {
            let new_cap = (self.glyph_capacities[buf_idx] as f64 * 1.5) as u64;
            let new_cap = new_cap.max(needed_bytes);
            self.glyph_buffers[buf_idx] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("acme glyph buffer"),
                size: new_cap,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.glyph_capacities[buf_idx] = new_cap;
            self.stats.buffer_grows += 1;
        }
    }
}
