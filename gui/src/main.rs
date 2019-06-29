//extern crate conrod;
#[macro_use]
extern crate conrod_core;
#[macro_use]
extern crate log;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate env_logger;
extern crate find_folder;
extern crate glium;

use api::client::Client;
use api::common::clipping::Clipping;
use api::server::Response;
use glium::Surface;

pub struct GliumDisplayWrapper(pub glium::Display);

const WIN_W: u32 = 1000;
const WIN_H: u32 = 1000;
const API_CONNECT_ERROR: &str = "Could not connect to clipboard daemon";

impl conrod_winit::WinitWindow for GliumDisplayWrapper {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.0.gl_window().get_inner_size().map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        self.0.gl_window().get_hidpi_factor() as _
    }
}

widget_ids!(pub struct Ids { canvas, list_select });

fn main() {
    env_logger::init();
    let mut api_client = Client::new().expect(API_CONNECT_ERROR);
    if let Err(e) = api_client.request_clipping() {
        error!("could not request clippings to daemon: {}", e);
        return;
    }
    let rsp = api_client.read_msg();
    if rsp.is_err() {
        error!("could not read message from daemon: {}", rsp.unwrap_err());
        return;
    }
    let rsp = rsp.unwrap();
    let clippings = match rsp {
        Response::Ok => panic!("Response to clipping request should not be `Ok`"),
        Response::Clippings(clippings) => clippings,
    };

    let mut context = Context::new();

    // create event loop
    // see conrod_glium::support::eventloop
    let mut event_loop = EventLoop::new();
    'main: loop {
        for event in event_loop.next(&mut context.events_loop) {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod_winit::convert_event(event.clone(), &context.display) {
                context.ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        // instanciate the widgets
        instanciate_widgets(&mut context, &clippings);
        draw(&mut context);
    }
}

pub fn instanciate_widgets(context: &mut Context, clippings: &Vec<Clipping>) {
    use conrod_core::{widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
    let id_selected = context.id_selected;
    let ui = &mut context.ui.set_widgets();

    widget::Canvas::new()
        .color(conrod_core::color::BLUE)
        .set(context.ids.canvas, ui);

    // Instantiate the `ListSelect` widget.
    let num_items = clippings.len();
    let item_h = 30.0;
    let font_size = item_h as conrod_core::FontSize / 2;
    let (mut events, scrollbar) = widget::ListSelect::single(num_items)
        .flow_down()
        .item_size(item_h)
        .scrollbar_next_to()
        .scrollbar_color(conrod_core::color::LIGHT_CHARCOAL)
        .w_h(WIN_H as f64 - 40., WIN_W as f64 - 40.)
        .top_left_with_margins_on(context.ids.canvas, 40.0, 40.0)
        .set(context.ids.list_select, ui);

    // Handle the `ListSelect`s events.
    while let Some(event) = events.next(ui, |i| i == id_selected) {
        use conrod_core::widget::list_select::Event;
        match event {
            // For the `Item` events we instantiate the `List`'s items.
            Event::Item(item) => {
                let label = &clippings[item.i].0;
                let (color, label_color) = match item.i == id_selected {
                    true => (conrod_core::color::LIGHT_BLUE, conrod_core::color::YELLOW),
                    false => (conrod_core::color::LIGHT_GREY, conrod_core::color::BLACK),
                };
                let button = widget::Button::new()
                    .border(0.0)
                    .color(color)
                    .label(label)
                    .label_font_size(font_size)
                    .label_color(label_color);
                item.set(button, ui);
            }

            // The selection has changed.
            Event::Selection(selection) => context.id_selected = selection,

            // The remaining events indicate interactions with the `ListSelect` widget.
            event => info!("{:?}", &event),
        }
    }

    // Instantiate the scrollbar for the list.
    if let Some(s) = scrollbar {
        s.set(ui);
    }
}

pub fn draw(context: &mut Context) {
    if let Some(primitives) = context.ui.draw_if_changed() {
        context
            .renderer
            .fill(&context.display.0, primitives, &context.image_map);
        let mut target = context.display.0.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        context
            .renderer
            .draw(&context.display.0, &mut target, &context.image_map)
            .unwrap();
        target.finish().unwrap();
    }
}

pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(
        &mut self,
        events_loop: &mut glium::glutin::EventsLoop,
    ) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}

pub struct Context {
    pub events_loop: glium::glutin::EventsLoop,
    pub display: GliumDisplayWrapper,
    pub ui: conrod_core::Ui,
    pub renderer: conrod_glium::Renderer,
    pub image_map: conrod_core::image::Map<glium::texture::Texture2d>,
    pub ids: Ids,
    pub id_selected: usize,
}

impl Context {
    pub fn new() -> Self {
        let events_loop = glium::glutin::EventsLoop::new();
        // construct window with configuration options
        let window = glium::glutin::WindowBuilder::new()
            .with_decorations(false)
            .with_dimensions((WIN_W, WIN_H).into());
        let context = glium::glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop)
            .expect("Could not create glium display");
        let display = GliumDisplayWrapper(display);
        let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();
        let ids = Ids::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::Kids(5)
            .for_folder("assets")
            .expect("Could not find assets folder");
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        //TODO: error out
        ui.fonts.insert_from_file(font_path).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

        let renderer =
            conrod_glium::Renderer::new(&display.0).expect("Could not create glium renderer");
        let id_selected = 0;
        Context {
            events_loop,
            display,
            ui,
            renderer,
            image_map,
            ids,
            id_selected,
        }
    }
}
