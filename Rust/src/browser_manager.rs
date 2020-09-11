use crate::{Coords2d, Dimensions};
use std::borrow::Borrow;
use std::convert::TryFrom;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;
use web_sys::{HtmlCanvasElement, HtmlImageElement};

#[derive(Clone)]
pub struct BrowserManager {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl BrowserManager {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");

        let canvas_element = window
            .document()
            .expect("should have a document on window")
            .get_element_by_id("screensaver-canvas")
            .unwrap();

        let canvas = canvas_element.dyn_into::<HtmlCanvasElement>().unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let bm = BrowserManager { canvas, context };

        bm.on_screen_resize(); // set screen size at startup

        // cloneable is okay because the underlying contents
        // themselves are just object pointers
        let closure_bm = bm.clone();

        let cb = Closure::wrap(Box::new(move || {
            closure_bm.on_screen_resize();
        }) as Box<dyn FnMut()>);

        window
            .add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref())
            .expect("resize event should work");

        cb.forget(); // i want callback to live for entire program

        bm
    }

    pub fn get_dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.canvas.width(),
            height: self.canvas.height(),
        }
    }

    pub fn paint_background(&self) {
        self.context.set_fill_style(&JsValue::from_str("black"));
        let dim = self.get_dimensions();
        self.context
            .fill_rect(0f64, 0f64, dim.width as f64, dim.height as f64);
    }

    pub fn draw_image(&self, image: &HtmlImageElement, coords: &Coords2d, width: f64, height: f64) {
        self.context
            .draw_image_with_html_image_element_and_dw_and_dh(
                image, coords.0, coords.1, width, height,
            )
            .unwrap()
    }

    pub fn create_request_animation_frame_loop(&self, loop_fn: Box<dyn FnMut(i32)>) {
        // put this stuff into the browser manager
        let cb = Closure::wrap(loop_fn);

        let _ = web_sys::window()
            .expect("no global `window` exists")
            .request_animation_frame(cb.borrow().as_ref().unchecked_ref());

        cb.forget(); // should be okay, since loop is meant to run for whole program
    }

    fn on_screen_resize(&self) {
        let body = web_sys::window()
            .expect("no global `window` exists")
            .document()
            .expect("should have a document on window")
            .body()
            .expect("document should have a body");

        self.canvas
            .set_width(u32::try_from(body.offset_width()).unwrap_or_default());
        self.canvas
            .set_height(u32::try_from(body.offset_height()).unwrap_or_default());
    }
}
