use std::{
    fs::File,
    path::{Path, PathBuf},
    env::temp_dir,
    io::Write,
    error::Error,
};
use ascetic_vis::{
    Scene, Theme, Style, Stroke, Fill, Marker, Variation, Group, Crumb, Joint, TextLabel, Color, UnitPoint,
    kurbo::{Rect, Circle, Arc, BezPath, PathEl},
    backend::{
        usvg::AsUsvgTree, usvg::Tree as UsvgTree, usvg::FitTo, usvg::Pixmap,
        usvg::render_to_pixmap, svg::ToSvg,
    },
};

const SCENE_NAME: &str = "scene";

fn roundabout_theme() -> Theme {
    let frame_stops = vec![Color::WHITE, Color::rgb8(0xd0, 0xd0, 0xd0)];
    let node_gradient_stops = vec![Color::WHITE, Color::rgb8(0, 0x60, 0)];
    let node_dark_gradient_stops = vec![Color::BLACK, Color::rgb8(0, 0x80, 0xff)];
    let token_gradient_stops = vec![Color::rgb8(0x80, 0, 0x80), Color::rgb8(0xff, 0, 0)];
    let token_dark_gradient_stops = vec![Color::BLACK, Color::rgb8(0xff, 0, 0xff)];

    let linear_gradients =
        vec![("frame", UnitPoint::TOP, UnitPoint::BOTTOM, frame_stops.as_slice())];

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
        ("line-thin", Stroke::new().with_brush(Color::BLACK).with_width(1.5)),
    ];

    let fills = vec![
        ("frame", Fill::Linear("frame".into())),
        ("node", Fill::Radial("node".into())),
        ("token", Fill::Radial("token".into())),
        ("black", Fill::Color(Color::BLACK)),
    ];

    let dark_strokes =
        vec![("node", Stroke::new().with_brush(Color::rgb8(0, 0x60, 0xff)).with_width(3.0))];

    let dark_fills = vec![
        (SCENE_NAME, Fill::Color(Color::BLACK)),
        ("node", Fill::Radial("node-dark".into())),
        ("token", Fill::Radial("token-dark".into())),
        ("black", Fill::Color(Color::WHITE)),
    ];

    let variations =
        vec![("dark", Variation::new().with_strokes(dark_strokes).with_fills(dark_fills))];

    let styles = vec![
        ("frame", Style::new().with_named_fill("frame").with_named_stroke("frame")),
        ("node", Style::new().with_named_fill("node").with_named_stroke("node")),
        ("token", Style::new().with_named_fill("token")),
        ("line-thick", Style::new().with_named_stroke("line-thick")),
        ("line-thin", Style::new().with_named_stroke("line-thin")),
        ("arrow1", Style::new().with_named_stroke("line-thin").with_named_end_marker("arrowhead1")),
        ("head1", Style::new().with_named_fill("black")),
    ];

    let markers = vec![(
        "arrowhead1",
        Marker::new(Crumb::Path(BezPath::from_vec(vec![
            PathEl::MoveTo((0.0, 0.0).into()),
            PathEl::LineTo((0.0, 7.0).into()),
            PathEl::LineTo((6.0, 3.5).into()),
            PathEl::ClosePath,
        ])))
        .with_size(12.0, 7.0)
        .with_refxy(0.0, 3.5)
        .with_named_style("head1"),
    )];

    Theme::new()
        .with_gradients(linear_gradients, radial_gradients)
        .with_markers(markers)
        .with_strokes(strokes)
        .with_fills(fills)
        .with_variations(variations)
        .with_styles(styles)
}

fn roundabout_scene(theme: &Theme) -> Scene {
    let mut scene = Scene::new((1000., 1000.));

    let node_positions = vec![
        (200.0, 400.0),
        (200.0, 600.0),
        (400.0, 200.0),
        (400.0, 400.0),
        (400.0, 600.0),
        (400.0, 800.0),
        (600.0, 200.0),
        (600.0, 400.0),
        (600.0, 600.0),
        (600.0, 800.0),
        (800.0, 400.0),
        (800.0, 600.0),
    ];
    let pin_positions = vec![
        (node_positions[0].0 - 135.0, node_positions[0].1 - 135.0),
        (node_positions[1].0 - 135.0, node_positions[1].1 + 135.0),
        (node_positions[10].0 + 135.0, node_positions[10].1 - 135.0),
        (node_positions[11].0 + 135.0, node_positions[11].1 + 135.0),
    ];
    let token_positions = vec![node_positions[0], node_positions[7], node_positions[9]];

    let node_style = theme.get("node");
    let nodes = scene.add_grouped_crumbs(
        node_positions
            .into_iter()
            .map(|(x, y)| (Crumb::Circle(Circle::new((x, y), 40.)), node_style)),
    );

    let pins = scene.add_grouped_crumbs(
        pin_positions.into_iter().map(|(x, y)| (Crumb::Pin(Circle::new((x, y), 40.)), None)),
    );

    let thick_style = theme.get("line-thick");
    let thin_style = theme.get("line-thin");
    let arrow_style = theme.get("arrow1");

    let lines = scene
        .join(nodes, nodes)
        .with_lines(arrow_style, [(2, 3), (3, 0), (1, 4), (4, 5), (9, 8), (8, 11), (10, 7), (7, 6)])
        .as_group(theme);

    let source_lines = scene
        .join(pins, nodes)
        .with_polylines(
            arrow_style,
            [(1, 1, [(20.0, 0.0), (-30.0, 10.0)]), (2, 10, [(-30.0, 10.0), (20.0, 0.0)])],
        )
        .as_group(theme);

    let sink_lines = scene
        .join(nodes, pins)
        .with_polylines(
            arrow_style,
            [(0, 0, [(0.0, -20.0), (10.0, 30.0)]), (11, 3, [(10.0, 30.0), (0.0, -20.0)])],
        )
        .as_group(theme);

    let radius = 2f64.sqrt() * 100.0;
    let arcs = scene
        .join(nodes, nodes)
        .with_arcs(arrow_style, [(7, 3, radius), (3, 4, -radius), (4, 8, radius), (8, 7, -radius)])
        .as_group(theme);

    let quads = scene
        .join(nodes, nodes)
        .with_curves(
            arrow_style,
            [
                (0, 10, [(0.0, -680.0)]),
                (11, 10, [(270.0, 0.0)]),
                (11, 1, [(0.0, 680.0)]),
                (0, 1, [(-270.0, 0.0)]),
            ],
        )
        .as_group(theme);

    let cubics = scene
        .join(nodes, nodes)
        .with_curves(
            arrow_style,
            [(6, 2, [(-20.0, -200.0), (20.0, 200.0)]), (5, 9, [(20.0, 200.0), (-20.0, -200.0)])],
        )
        .as_group(theme);

    let token_style = theme.get("token");
    let tokens = scene.add_grouped_crumbs(
        token_positions
            .into_iter()
            .map(|(x, y)| (Crumb::Circle(Circle::new((x, y), 10.)), token_style)),
    );

    let radius = 500.0;
    let frame = scene.add_grouped_crumbs([
        (Crumb::Rect(Rect::new(0., 0., 1000., 1000.)), theme.get("frame")),
        (
            Crumb::Arc(Arc {
                center:      (500.0, 500.0).into(),
                radii:       (radius, radius).into(),
                start_angle: 0.0,
                sweep_angle: std::f64::consts::PI,
                x_rotation:  0.0,
            }),
            thin_style,
        ),
        (
            Crumb::Arc(Arc {
                center:      (500.0, 500.0).into(),
                radii:       (radius, radius).into(),
                start_angle: std::f64::consts::PI,
                sweep_angle: std::f64::consts::PI,
                x_rotation:  0.0,
            }),
            thin_style,
        ),
        (
            Crumb::Circle(Circle { center: (500.0, 500.0).into(), radius: radius - 10.0 }),
            thick_style,
        ),
    ]);

    let labels = scene.add_grouped_crumbs([
        (
            Crumb::Label(TextLabel::new("A text label".to_string()).with_xy(100.0, 100.0)),
            thin_style,
        ),
    ]);

    scene.add_root(Group::from_groups([
        labels,
        tokens,
        nodes,
        lines,
        source_lines,
        sink_lines,
        arcs,
        quads,
        cubics,
        frame,
        pins,
    ]));

    scene
}

#[inline]
fn done_in_micros(start_time: Option<std::time::Instant>) {
    if let Some(elapsed) = start_time.map(|t| t.elapsed().as_micros()) {
        eprintln!(" Done ({} us).", elapsed);
    }
}

#[derive(Debug)]
struct App {
    png_path:         PathBuf,
    svg_path:         Option<PathBuf>,
    out_size:         (f64, f64),
    out_margin:       (f64, f64),
    png_color_type:   png::ColorType,
    png_bit_depth:    png::BitDepth,
    png_compression:  png::Compression,
    png_filter:       png::FilterType,
    theme_variation:  Option<String>,
    variation_amount: Option<f64>,
    #[allow(dead_code)]
    verbosity:        u32,
}

impl App {
    const DEFAULT_PNG_PATH: &'static str = "test.png";
    const DEFAULT_OUT_SIZE: (f64, f64) = (800., 450.);
    const DEFAULT_OUT_MARGIN: (f64, f64) = (10., 10.);
    const DEFAULT_PNG_COLOR_TYPE: png::ColorType = png::ColorType::RGBA;
    const DEFAULT_PNG_BIT_DEPTH: png::BitDepth = png::BitDepth::Eight;
    const DEFAULT_PNG_COMPRESSION: png::Compression = png::Compression::Fast;
    const DEFAULT_PNG_FILTER: png::FilterType = png::FilterType::NoFilter;

    fn new() -> Result<Self, Box<dyn Error>> {
        let mut png_path = None;
        let mut svg_path = None;
        let mut with_svg = false;
        let mut out_size = Self::DEFAULT_OUT_SIZE;
        let out_margin = Self::DEFAULT_OUT_MARGIN;
        let png_color_type = Self::DEFAULT_PNG_COLOR_TYPE;
        let png_bit_depth = Self::DEFAULT_PNG_BIT_DEPTH;
        let png_compression = Self::DEFAULT_PNG_COMPRESSION;
        let png_filter = Self::DEFAULT_PNG_FILTER;
        let mut theme_variation = None;
        let mut variation_amount = None;
        let mut verbosity = 0;

        for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
            match next_arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                "-vvv" => verbosity += 3,
                "--with-svg" => with_svg = true,
                "-w" | "-h" | "--svg" | "--theme" | "--amount" => {}
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        match prev_arg.as_str() {
                            "-w" => out_size.0 = arg.parse()?,
                            "-h" => out_size.1 = arg.parse()?,
                            "--svg" => svg_path = Some(PathBuf::from(arg)),
                            "--theme" => theme_variation = Some(next_arg),
                            "--amount" => variation_amount = Some(arg.parse()?),
                            _ => png_path = Some(PathBuf::from(arg)),
                        }
                    }
                }
            }
        }

        let png_path = png_path.unwrap_or_else(|| {
            let mut path = temp_dir();
            path.push(Self::DEFAULT_PNG_PATH);
            if verbosity > 0 {
                eprintln!("[WARN] Unspecified PNG output path; using \"{}\".", path.display());
            }
            path
        });

        if with_svg && svg_path.is_none() {
            svg_path = Some(png_path.clone().with_extension("svg"));
        }

        Ok(App {
            png_path,
            svg_path,
            out_size,
            out_margin,
            png_color_type,
            png_bit_depth,
            png_compression,
            png_filter,
            theme_variation,
            variation_amount,
            verbosity,
        })
    }

    #[inline]
    fn start(&self, message: &str) -> Option<std::time::Instant> {
        if self.verbosity > 0 {
            eprint!("{}", message);
            Some(std::time::Instant::now())
        } else {
            None
        }
    }

    fn render_to_svg(&self, scene: &Scene, theme: &Theme) -> Result<Option<&Path>, Box<dyn Error>> {
        if let Some(ref svg_path) = self.svg_path {
            let start_time = self.start("Rendering to svg...");
            let svg = scene.to_svg(theme, self.out_size, self.out_margin)?;
            done_in_micros(start_time);

            let start_time =
                self.start(format!("Saving scene to \"{}\"...", svg_path.display()).as_str());
            let mut svg_file = File::create(svg_path)?;
            svg_file.write_all(&svg.into_bytes())?;
            done_in_micros(start_time);

            Ok(Some(svg_path.as_path()))
        } else {
            Ok(None)
        }
    }

    fn render_to_png(&self, scene: &Scene, theme: &Theme) -> Result<&Path, Box<dyn Error>> {
        let start_time = self.start("Rendering to usvg...");
        let rtree = scene.as_usvg_tree(theme, self.out_size, self.out_margin);
        done_in_micros(start_time);

        self.save_bitmap_image(&rtree)
    }

    fn save_bitmap_image(&self, rtree: &UsvgTree) -> Result<&Path, Box<dyn Error>> {
        let start_time = self.start("Rendering to bitmap...");
        let pixmap_size = rtree.svg_node().size.to_screen_size();
        let mut pixmap =
            Pixmap::new(pixmap_size.width(), pixmap_size.height()).expect("pixmap creation");
        render_to_pixmap(rtree, FitTo::Original, pixmap.as_mut()).expect("pixmap rendering");
        done_in_micros(start_time);

        let start_time = self
            .start(format!("Writing image data to \"{}\"...", self.png_path.display()).as_str());
        pixmap.save_png(&self.png_path)?;
        done_in_micros(start_time);

        Ok(self.png_path.as_path())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    let mut theme = roundabout_theme();
    let scene = roundabout_scene(&theme);

    if app.verbosity > 1 {
        if app.verbosity > 2 {
            eprintln!("{:?}\n", app);
        }
        eprintln!("{:?}", scene);
    }

    if let Some(ref variation) = app.theme_variation {
        if let Some(amount) = app.variation_amount {
            theme.start_variation(Some(variation), 1);
            theme.step_variation(amount);
        } else {
            theme.use_variation(Some(variation));
        }
    }

    if app.verbosity > 2 {
        eprintln!("\n{:?}", theme);
    }

    let svg_path = app.render_to_svg(&scene, &theme)?;
    let png_path = app.render_to_png(&scene, &theme)?;

    if let Some(svg_path) = svg_path {
        println!("{} {}", svg_path.display(), png_path.display());
    } else {
        println!("{}", png_path.display());
    }

    Ok(())
}
