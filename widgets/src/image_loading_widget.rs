use crate::{
        makepad_image_formats::*,
        makepad_draw::*,
    };

pub trait ImageLoadingWidget {
    fn image_filename(&self) -> &LiveDependency;
    fn texture(&mut self) -> &mut Option<Texture>;

    fn after_apply_for_image_loading_widget(&mut self, cx: &mut Cx, _from: ApplyFrom, index: usize, nodes: &[LiveNode]) {
        if self.texture().is_none() {
            let filename = self.image_filename();
            let image_path = filename.as_str();

            let buffer = {
                if image_path.len() > 0 {
                    if let Some(buffer) = cx.image_cache.get(image_path) {
                        Some(buffer)
                    } else {
                        if let Some(buffer) =
                            Self::load_image_dependency(cx, image_path, index, nodes)
                        {
                            cx.image_cache
                                .put(image_path, buffer.clone());
                            Some(buffer)
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(mut buffer) = buffer {
                self.create_texture_from_image(cx, &mut buffer);
            }
        }
    }

    fn create_texture_from_image(&mut self, cx: &mut Cx, image_buffer: &mut ImageBuffer) {
        if self.texture().is_none() {
            let texture = self.texture();
            *texture = Some(Texture::new(cx));
        }
        if let Some(texture) = &mut self.texture() {
            texture.set_desc(cx, TextureDesc {
                format: TextureFormat::ImageBGRA,
                width: Some(image_buffer.width),
                height: Some(image_buffer.height),
            });
            texture.swap_image_u32(cx, &mut image_buffer.data);
        }
    }

    fn load_image_dependency(cx: &mut Cx, image_path: &str, index: usize, nodes: &[LiveNode]) -> Option<ImageBuffer> {
        match cx.get_dependency(image_path) {
            Ok(data) => {
                if image_path.ends_with(".jpg") {
                    match jpeg::decode(data) {
                        Ok(image) => {
                            Some(image)
                        }
                        Err(err) => {
                            cx.apply_image_decoding_failed(live_error_origin!(), index, nodes, image_path, &err);
                            None
                        }
                    }
                }
                else if image_path.ends_with(".png") {
                    match png::decode(data) {
                        Ok(image) => {
                            Some(image)
                        }
                        Err(err) => {
                            cx.apply_image_decoding_failed(live_error_origin!(), index, nodes, image_path, &err);
                            None
                        }
                    }
                }
                else {
                    cx.apply_image_type_not_supported(live_error_origin!(), index, nodes, image_path);
                    None
                }
            }
            Err(err) => {
                cx.apply_resource_not_loaded(live_error_origin!(), index, nodes, image_path, &err);
                None
            }
        }
    }
}
