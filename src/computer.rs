use crate::{gpu_interface::GPUInterface, math::IVec2};

pub trait Computer {
    fn get_read_write(&self) -> (usize, usize) {
        let mut read = 0;
        let mut write = 1;
        if self.get_current_frame() % 2 == 1 {
            read = 1;
            write = 0;
        }
        (read, write)
    }

    fn get_current_texture(&self) -> &wgpu::Texture {
        &self.get_textures()[self.get_read_write().1]
    }

    fn get_current_frame(&self) -> usize;

    fn get_textures(&self) -> &[wgpu::Texture; 2];

    fn get_size(&self) -> IVec2;

    fn step(&mut self, gpu: &GPUInterface);

    fn compute_work_group_count(
        &self,
        (width, height): (u32, u32),
        (workgroup_width, workgroup_height): (u32, u32),
    ) -> (u32, u32) {
        let x = (width + workgroup_width - 1) / workgroup_width;
        let y = (height + workgroup_height - 1) / workgroup_height;

        (x, y)
    }
}
