use std::iter::IntoIterator;
use tracing::error;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::EventLoopExtUnix,
    window::WindowBuilder,
};
use ascetic_vis::{
    Scene, Theme, Style, Stroke, Fill, Variation, Group, CrumbItem,
    Color, UnitPoint,
    kurbo::{Line, Rect, Circle, TranslateScale},
};
use ascetic_rut::{Gui, Logger};

const SCENE_NAME: &str = "scene";

fn roundabout_theme() -> Theme {
    let frame_stops = vec![Color::WHITE, Color::BLACK];
    let node_gradient_stops = vec![Color::WHITE, Color::rgb8(0, 0x60, 0)];
    let node_dark_gradient_stops = vec![Color::BLACK, Color::rgb8(0, 0x80, 0xff)];
    let token_gradient_stops = vec![Color::rgb8(0x80, 0, 0x80), Color::rgb8(0xff, 0, 0)];
    let token_dark_gradient_stops = vec![Color::BLACK, Color::rgb8(0xff, 0, 0xff)];

    let linear_gradients = vec![("frame", UnitPoint::TOP, UnitPoint::BOTTOM, frame_stops.as_slice())];

    let radial_gradients = vec![
        ("node", 1., node_gradient_stops.as_slice()),
        ("node-dark", 1., node_dark_gradient_stops.as_slice()),
        ("token", 1., token_gradient_stops.as_slice()),
        ("token-dark", 1., token_dark_gradient_stops.as_slice()),
    ];

    let strokes = vec![
        ("frame", Stroke::new().with_brush(Color::BLACK).with_width(0.5)),
        ("node", Stroke::new().with_brush(Color::rgb8(0, 0x80, 0)).with_width(3.0)),
        ("line-thick", Stroke::new().with_brush(Color::BLACK).with_width(3.0)),
        ("line-thin", Stroke::new().with_brush(Color::BLACK).with_width(0.5)),
    ];

    let fills = vec![
        ("frame", Fill::Linear("frame".into())),
        ("node", Fill::Radial("node".into())),
        ("token", Fill::Radial("token".into())),
    ];

    let dark_strokes =
        vec![("node", Stroke::new().with_brush(Color::rgb8(0, 0x60, 0xff)).with_width(3.0))];

    let dark_fills = vec![
        (SCENE_NAME, Fill::Color(Color::BLACK)),
        ("node", Fill::Radial("node-dark".into())),
        ("token", Fill::Radial("token-dark".into())),
    ];

    let variations =
        vec![("dark", Variation::new().with_strokes(dark_strokes).with_fills(dark_fills))];

    let styles = vec![
        ("frame", Style::new().with_named_fill("frame").with_named_stroke("frame")),
        ("node", Style::new().with_named_fill("node").with_named_stroke("node")),
        ("token", Style::new().with_named_fill("token")),
        ("line-thick", Style::new().with_named_stroke("line-thick")),
        ("line-thin", Style::new().with_named_stroke("line-thin")),
    ];

    Theme::new()
        .with_gradients(linear_gradients, radial_gradients)
        .with_strokes(strokes)
        .with_fills(fills)
        .with_variations(variations)
        .with_styles(styles)
}

fn roundabout_scene(theme: &Theme) -> Scene {
    let mut scene = Scene::new((1000., 1000.));

    let frame = scene.add_rect(Rect::new(0., 0., 1000., 1000.));

    let node = scene.add_circle(Circle::new((0., 0.), 40.));
    let node_style = theme.get("node");

    let token = scene.add_circle(Circle::new((0., 0.), 10.));
    let token_style = theme.get("token");

    let node_positions = vec![
        (200.0, 400.0),
        (200.0, 600.0),
        (400.0, 200.0),
        (600.0, 200.0),
        (800.0, 400.0),
        (800.0, 600.0),
        (400.0, 800.0),
        (600.0, 800.0),
        (400.0, 400.0),
        (400.0, 600.0),
        (600.0, 400.0),
        (600.0, 600.0),
    ];
    let token_positions = vec![node_positions[0], node_positions[7]];

    let thick_style = theme.get("line-thick");
    let thin_style = theme.get("line-thin");

    let lines = scene.add_grouped_lines(vec![
        (Line::new((0., 500.), (250., 0.)), thick_style),
        (Line::new((0., 500.), (250., 1000.)), thin_style),
        (Line::new((250., 1000.), (250., 0.)), thin_style),
    ]);

    let nodes = scene.add_grouped_crumb_items(
        node_positions
            .into_iter()
            .map(|(x, y)| CrumbItem(node, TranslateScale::translate((x, y).into()), node_style)),
    );

    let tokens = scene.add_grouped_crumb_items(
        token_positions
            .into_iter()
            .map(|(x, y)| CrumbItem(token, TranslateScale::translate((x, y).into()), token_style)),
    );

    scene.add_root(
        Group::from_groups(vec![tokens, nodes, lines])
            .with_crumb(frame, theme.get("frame"))
    );

    scene
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win_width: u32 = 800;
    let mut win_height: u32 = 600;
    let mut verbosity = 0;

    for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
        match next_arg.as_str() {
            "-v" => verbosity += 1,
            "-vv" => verbosity += 2,
            "-vvv" => verbosity += 3,
            "-w" | "-h" => {}
            arg => {
                if arg.starts_with('-') {
                    panic!("ERROR: Invalid CLI option \"{}\"", arg)
                } else {
                    match prev_arg.as_str() {
                        "-w" => win_width = arg.parse()?,
                        "-h" => win_height = arg.parse()?,
                        _ => panic!("ERROR: Invalid CLI argument \"{}\"", arg),
                    }
                }
            }
        }
    }

    let mut logger = Logger::new("error")?;
    logger.set_crate_filter(match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    })?;
    logger.install()?;

    let event_loop: EventLoop<()> = EventLoop::new_x11().unwrap();
    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(win_width, win_height))
        .with_position(PhysicalPosition::new(100, 100))
        .with_title("roundabout")
        .with_resizable(true)
        .with_decorations(true)
        .with_transparent(true);
    let mut gui = Gui::new(&event_loop, &window_builder)?;
    let mut theme = roundabout_theme();
    let mut scene = roundabout_scene(&theme);

    event_loop.run(move |event, _, control_flow| match gui.update(event, &mut scene, &mut theme) {
        Ok(done) => *control_flow = if done { ControlFlow::Exit } else { ControlFlow::Wait },
        Err(err) => {
            use ascetic_rut::Error::*;
            match err {
                Fatal(err) => {
                    error!("{}", err);
                    *control_flow = ControlFlow::Exit;
                    return
                }
                WinitFailure(err) => error!("{}", err),
                SwapChainFailure(err) => error!("{}", err),
                PixelsFailure(err) => error!("{}", err),
                err => error!("{}", err),
            }
            *control_flow = ControlFlow::Wait;
        }
    });
}
