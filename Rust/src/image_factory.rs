use crate::color::Color;

use futures::future::join_all;
use js_sys::Promise;
use std::collections::HashMap;
use std::include_str;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlImageElement;

const SVG_CONTENT: &str = include_str!("../../Windows_Logo_1995.svg");

pub struct ImageFactory {
    image_map: HashMap<Color, HtmlImageElement>,
}

// Used to load/cache the Windows Logo SVG in the colors provided by the caller.
// Once it has been initialized, getting the tinted image is a simple hashmap lookup.
impl ImageFactory {
    pub async fn init(colors: &[Color]) -> Self {
        let img_futures = colors.iter().map(|c| Self::load_img_with_color(*c));
        let combined = join_all(img_futures).await;

        let mut image_map = HashMap::new();

        for (color, image) in colors.iter().zip(combined.into_iter()) {
            image_map.insert(*color, image);
        }

        ImageFactory { image_map }
    }

    pub fn get_tinted_image(&self, color: Color) -> &HtmlImageElement {
        self.image_map
            .get(&color)
            .expect("Color must be initialized at startup")
    }

    async fn load_img_with_color(color: Color) -> HtmlImageElement {
        let img = HtmlImageElement::new().unwrap();

        // I could not find an easy way to draw an SVG image on the canvas with a color tint,
        // so this hack is the easiest workaround. Performs a text replace in the SVG DOM to replace
        // the color black with the color of our choosing
        let tinted_svg = SVG_CONTENT.replace("#000000", color.as_web_color());
        let encoded = js_sys::encode_uri_component(&tinted_svg);

        img.set_src(&format!(
            "data:image/svg+xml;charset=utf-8,{}",
            encoded.as_string().unwrap()
        ));

        let img_promise = Promise::new(&mut |resolve, _reject| {
            img.set_onload(Some(&resolve));
        });

        let img_fut: JsFuture = img_promise.into();
        img_fut.await.unwrap();

        img
    }
}
