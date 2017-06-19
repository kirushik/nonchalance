#[macro_use]
extern crate conrod;
use conrod::{widget, Positionable, Sizeable, Labelable, Widget};
use conrod::text::FontCollection;
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};

extern crate ttf_noto_sans;
extern crate ws;
extern crate url;
use url::Url;

use std::thread;
use std::sync::mpsc::channel;

mod websockets;
use websockets::{WSHandler, GuiCallbackChannel};

pub fn main() {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 200;

    // Build the window.
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("Nonchalance, SecureLogin application")
        .with_multisampling(4)
        .build_glium()
        .unwrap();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    let mut url: String = std::env::args().nth(1).unwrap_or("https://ya.ru".into());
    let mut callback: Option<GuiCallbackChannel> = None;

    // Generate the widget identifiers.
    widget_ids!(struct Ids { button, text_box });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    ui.fonts.insert(FontCollection::from_bytes(ttf_noto_sans::REGULAR).into_font().expect("failed to load Noto font"));

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    let (url_request_sender, url_request_receiver) = channel::<Option<(Url, GuiCallbackChannel)>>();
    thread::spawn(move || {
        ws::listen("127.0.0.1:3101", |out| WSHandler::new(out, url_request_sender.clone()) ).expect("Failed to create websocket listener on 3101!");
    });


    // Poll events from the window.
    let mut last_update = std::time::Instant::now();
    let mut ui_needs_update = true;

    'main: loop {

        if let Some(option) = url_request_receiver.try_recv().ok() {
            if let Some((new_url, callback_channel)) = option {
                url = new_url.into_string();
                callback = Some(callback_channel);
            } else {
                callback = None;
            }
        }

        // we don't want to loop any faster than 60 fps, so wait until it has been at least
        // 16ms since the last yield.
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events: Vec<_> = display.poll_events().collect();

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !ui_needs_update {
            events.extend(display.wait_events().next());
        }

        // Reset the needs_update flag and time this update.
        ui_needs_update = false;
        last_update = std::time::Instant::now();

        // Handle all events.
        for event in events {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                ui.handle_event(event);
                ui_needs_update = true;
            }

            match event {
                // Break from the loop upon `Escape`.
                glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                    glium::glutin::Event::Closed =>
                        break 'main,
                _ => {}
            }
        }

        // Instantiate all widgets in the GUI.
        {
            let ui = &mut ui.set_widgets();

            for _click in widget::Button::new()
                .middle_of(ui.window)
                .w_h(80.0, 80.0)
                .label("GO")
                .set(ids.button, ui) {
                    callback.clone().map(|tx| tx.send("Приехали!".into()));
                }

            for event in widget::TextBox::new(url.as_str())
                                        .left_justify()
                                        .mid_top_of(ui.window)
                                        .padded_w_of(ui.window, 20.0)
                                        // .border(1.0)
                                        // .border_color(conrod::color::WHITE)
                                        // .color(conrod::color::WHITE)
                                        .font_size(16)
                                        .set(ids.text_box, ui)
                                        {
                                            use conrod::widget::text_box::Event::*;
                                            match event {
                                                Update(new_url) => url = new_url,
                                                Enter => {}
                                            }
                                        }
        }

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
