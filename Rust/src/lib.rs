use js_sys::Math;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod browser_manager;
mod color;
mod image_factory;

use browser_manager::BrowserManager;
use color::{Color, ALL_COLORS};
use image_factory::ImageFactory;

#[derive(Debug)]
pub struct Coords2d(f64, f64);

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    width: u32,
    height: u32,
}

impl Dimensions {
    fn center(&self) -> Coords2d {
        let center_x = (self.width / 2) as f64;
        let center_y = (self.height / 2) as f64;

        Coords2d(center_x, center_y)
    }
}

// width, height should be determined in one place, not both in render as well as update
#[derive(Debug)]
struct WindowsLogo {
    position: Coords2d,
    dimensions: Dimensions,
    speed: f64,
    color: Color,
}

impl WindowsLogo {
    pub fn create_random_logo(max_dimension: Dimensions, speed: f64) -> Self {
        // setting dummy placeholder values
        let mut logo = WindowsLogo {
            position: Coords2d(0f64, 0f64),
            dimensions: Dimensions {
                width: 0,
                height: 0,
            },
            speed,
            color: Color::White,
        };

        logo.recycle_with_random_values(max_dimension);

        logo
    }

    pub fn recycle_with_random_values(&mut self, max_dimension: Dimensions) {
        let random_index = Math::random() * ALL_COLORS.len() as f64;
        let random_color = ALL_COLORS[random_index.floor() as usize];

        // every once in a while, bias towards the center
        let should_bias = (Math::random() * 3f64).floor() == 0f64;

        let max_x = max_dimension.width as f64;
        let max_y = max_dimension.height as f64;

        let random_position = if should_bias {
            // random position bias could be better cleaner
            let fourth_x = max_x / 4f64;
            let fourth_y = max_y / 4f64;

            let x = fourth_x + Math::random() * (max_x - (fourth_x * 2f64));
            let y = fourth_y + Math::random() * (max_y - (fourth_y * 2f64));
            Coords2d(x, y)
        } else {
            let x = Math::random() * max_x;
            let y = Math::random() * max_y;
            Coords2d(x, y)
        };

        self.position = random_position;
        self.color = random_color;

        // can set dimensions to none, since it will get modified in the update_state method
        self.dimensions = Dimensions {
            width: 0,
            height: 0,
        };
    }
}

struct ScreenSaver {
    browser: BrowserManager,
    image_factory: ImageFactory,

    // is it possible to find a way to make these not Cells?
    logos: RefCell<Vec<WindowsLogo>>,
    last_elapsed: Cell<i32>,
}

impl ScreenSaver {
    pub fn new(
        browser: BrowserManager,
        image_factory: ImageFactory,
        number_of_windows: u8,
        window_speed: f64,
    ) -> Self {
        let dimensions = browser.get_dimensions();
        let logos = (0..number_of_windows)
            .map(|_| WindowsLogo::create_random_logo(dimensions, window_speed));

        ScreenSaver {
            browser,
            image_factory,
            logos: RefCell::new(logos.collect()),
            last_elapsed: Cell::new(0),
        }
    }

    pub fn start(self) {
        let rc = Rc::new(self);
        rc.advance_world(0);
    }

    // would prefer not to need to use Rc, but unsure how to keep a self reference in the request animation frame callback
    fn advance_world(self: Rc<Self>, elapsed_time: i32) {
        const APPROX_FRAMES_PER_SECOND: i32 = 60;
        const TIME_SLICE: i32 = 1000 / APPROX_FRAMES_PER_SECOND;

        let time_since_last_tick = elapsed_time - self.last_elapsed.get();

        if time_since_last_tick >= TIME_SLICE {
            self.last_elapsed.set(elapsed_time);

            self.update_state(time_since_last_tick);
            self.render_content();
        }

        let rc = self.clone();

        let cb = Box::new(move |x| {
            rc.clone().advance_world(x);
        }) as Box<dyn FnMut(i32)>;

        self.browser.create_request_animation_frame_loop(cb);
    }

    fn update_state(&self, time_since_last_tick: i32) {
        const MAGIC_SIZE_CONSTANT: f64 = 0.22; // just seems to work

        let dimensions = self.browser.get_dimensions();
        let Coords2d(cx, cy) = dimensions.center();

        let time_multiplier = time_since_last_tick as f64 / 1000f64;

        for logo in &mut self.logos.borrow_mut().iter_mut() {
            let Coords2d(x, y) = logo.position;

            logo.position.0 += ((x - cx) * logo.speed) * time_multiplier;
            logo.position.1 += ((y - cy) * logo.speed) * time_multiplier;

            let width = MAGIC_SIZE_CONSTANT * (x - cx).abs();
            let height = MAGIC_SIZE_CONSTANT * (y - cy).abs();

            logo.dimensions = Dimensions {
                width: width as u32,
                height: height as u32,
            };

            let Coords2d(x, y) = logo.position;

            if x > dimensions.width as f64 + width
                || x < -width
                || y > dimensions.height as f64 + height
                || y < -height
            {
                WindowsLogo::recycle_with_random_values(logo, dimensions);
            }
        }
    }

    fn render_content(&self) {
        self.browser.paint_background();

        for logo in self.logos.borrow().iter() {
            let img = self.image_factory.get_tinted_image(logo.color);

            self.browser.draw_image(
                img,
                &logo.position,
                logo.dimensions.width as f64,
                logo.dimensions.height as f64,
            );
        }
    }
}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let _ = future_to_promise(async {
        let factory = ImageFactory::init(&ALL_COLORS).await;
        let browser = BrowserManager::new();

        let number_windows: u8 = 20;
        let magic_velocity_number: f64 = 1.2; // just trial and error to the value that I think looked good

        let screensaver = ScreenSaver::new(browser, factory, number_windows, magic_velocity_number);

        screensaver.start();

        Ok(JsValue::NULL)
    });
    Ok(())
}
