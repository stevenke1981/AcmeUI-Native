//! Keeps the CPU-side RGBA8 image set and the resident GPU textures in sync with a scene.

use std::collections::{HashMap, HashSet};

use acme_core::{Rgba8Image, Scene};

use crate::images::{
    ImageResourceKey, ImageUploadLayout, collect_scene_images, create_image_resource,
    retain_live_image_resources, validate_image_upload, write_rgba8_texture,
};

use super::Renderer;

impl Renderer {
    pub(super) fn update_live_scene_images(&mut self, scene: &Scene) {
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        let scene_images = collect_scene_images(scene);
        let mut live_images = HashMap::new();
        for (resource_key, image) in scene_images {
            match validate_image_upload(image.width(), image.height(), max_dimension) {
                Ok(_) => {
                    live_images.insert(resource_key, image.clone());
                }
                Err(error) => {
                    tracing::warn!(
                        key = image.key().get(),
                        revision = image.revision(),
                        ?error,
                        "rejected RGBA8 image upload"
                    );
                }
            }
        }
        self.live_images = live_images;
    }

    pub(super) fn sync_scene_images(&mut self) -> u64 {
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        let live: HashSet<ImageResourceKey> = self.live_images.keys().copied().collect();
        let valid: Vec<(ImageResourceKey, Rgba8Image, ImageUploadLayout)> = self
            .live_images
            .iter()
            .filter_map(|(key, image)| {
                validate_image_upload(image.width(), image.height(), max_dimension)
                    .ok()
                    .map(|layout| (*key, image.clone(), layout))
            })
            .collect();
        let mut uploaded = 0;
        for (resource_key, image, layout) in valid {
            if self.images.contains_key(&resource_key) {
                continue;
            }

            let reusable_key = self
                .images
                .iter()
                .find(|(existing_key, resource)| {
                    existing_key.0 == resource_key.0
                        && !live.contains(existing_key)
                        && resource.image.width() == image.width()
                        && resource.image.height() == image.height()
                })
                .map(|(existing_key, _)| *existing_key);
            if let Some(reusable_key) = reusable_key {
                let mut resource = self
                    .images
                    .remove(&reusable_key)
                    .expect("selected reusable image resource must exist");
                write_rgba8_texture(&self.queue, &resource._texture, &image, layout);
                resource.image = image.clone();
                self.images.insert(resource_key, resource);
            } else {
                let resource = create_image_resource(
                    &self.device,
                    &self.queue,
                    &self.texture_layout,
                    &self.image_sampler,
                    image.clone(),
                    layout,
                );
                self.images.insert(resource_key, resource);
            }
            uploaded += image.pixels().len() as u64;
        }
        retain_live_image_resources(&mut self.images, &live);
        uploaded
    }
}
