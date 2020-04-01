use std::{
    str::FromStr,
    ops::Range,
    fs,
    path::{Path, PathBuf},
    fmt,
    num::ParseFloatError,
    error::Error,
};
use roxmltree as xml;
use cssparser as css;
use crate::{PetItemKind, PetError};

#[derive(Debug)]
pub struct Scope {
    name:       String,
    byte_range: Range<usize>,
    text_range: Range<xml::TextPos>,
}

impl Scope {
    #[inline]
    fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    fn text_start(&self) -> xml::TextPos {
        self.text_range.start
    }
}

#[derive(Debug)]
pub enum PNMLError {
    InvalidXML(xml::Error),
    InvalidTag(Scope, String),
    MissingAttribute(Scope, String),
    MissingElement(Scope, String),
    MissingText(Scope, Scope),
    InvalidChildElement(Scope, String, Scope),
    UnknownTool(Scope, String, Scope),
    MissingToolName(Scope, String, Scope),
    InvalidAttributeValue(Scope, String, String),
    BadFloat(Scope, ParseFloatError),
    BadColor(Scope, String),
}

impl PNMLError {
    pub fn text_start(&self) -> xml::TextPos {
        use PNMLError::*;

        match self {
            InvalidXML(err) => err.pos(),
            InvalidTag(ref scope, _)
            | MissingAttribute(ref scope, _)
            | MissingElement(ref scope, _)
            | MissingText(_, ref scope)
            | InvalidChildElement(_, _, ref scope)
            | UnknownTool(_, _, ref scope)
            | MissingToolName(_, _, ref scope)
            | InvalidAttributeValue(ref scope, ..)
            | BadFloat(ref scope, _)
            | BadColor(ref scope, _) => scope.text_start(),
        }
    }

    fn invalid_tag<'a, 'input, S: AsRef<str>>(node: &xml::Node<'a, 'input>, expected: S) -> Self {
        PNMLError::InvalidTag(node.get_scope(), expected.as_ref().into())
    }

    fn invalid_child_element<'a, 'input>(
        parent: &xml::Node<'a, 'input>,
        child: &xml::Node<'a, 'input>,
    ) -> Self {
        PNMLError::InvalidChildElement(
            parent.get_scope(),
            parent.get_or_make_id(),
            child.get_scope(),
        )
    }

    fn unknown_or_missing_tool<'a, 'input>(
        parent: &xml::Node<'a, 'input>,
        child: &xml::Node<'a, 'input>,
    ) -> Self {
        if let Some(tool_name) = child.get_attribute("tool") {
            PNMLError::UnknownTool(
                parent.get_scope(),
                parent.get_or_make_id(),
                child.get_scope_renamed(tool_name),
            )
        } else {
            PNMLError::MissingToolName(
                parent.get_scope(),
                parent.get_or_make_id(),
                child.get_scope(),
            )
        }
    }
}

impl From<xml::Error> for PNMLError {
    #[inline]
    fn from(err: xml::Error) -> Self {
        PNMLError::InvalidXML(err)
    }
}

impl fmt::Display for PNMLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PNMLError::*;

        match self {
            InvalidXML(err) => err.fmt(f),
            InvalidTag(ref scope, expected) => {
                write!(f, "Unexpected tag name <{}> instead of <{}>", scope.name(), expected)
            }
            MissingAttribute(ref scope, attr_name) => {
                write!(f, "Attribute \"{}\" missing in element <{}>", attr_name, scope.name())
            }
            MissingElement(ref parent_scope, child_name) => write!(
                f,
                "Element <{}> missing under element <{}>",
                child_name,
                parent_scope.name()
            ),
            MissingText(ref parent_scope, ref child_scope) => write!(
                f,
                "Missing text of element <{}> under element <{}>",
                child_scope.name(),
                parent_scope.name()
            ),
            InvalidChildElement(ref parent_scope, parent_id, ref child_scope) => write!(
                f,
                "Unexpected child element <{}> of <{}> \"{}\"",
                child_scope.name(),
                parent_scope.name(),
                parent_id
            ),
            UnknownTool(ref parent_scope, parent_id, ref tool_scope) => write!(
                f,
                "Element specific to unknown tool \"{}\" of <{}> \"{}\"",
                tool_scope.name(),
                parent_scope.name(),
                parent_id
            ),
            MissingToolName(ref parent_scope, parent_id, _) => write!(
                f,
                "Unspecified tool for tool-specific element of <{}> \"{}\"",
                parent_scope.name(),
                parent_id
            ),
            InvalidAttributeValue(ref scope, name, value) => {
                write!(f, "Unexpected {} \"{}\" in element <{}>", name, value, scope.name())
            }
            BadFloat(ref scope, err) => {
                write!(f, "Float parsing failed in element <{}>: {}", scope.name(), err)
            }
            BadColor(ref scope, err) => {
                write!(f, "Color parsing failed in element <{}>: {}", scope.name(), err)
            }
        }
    }
}

impl Error for PNMLError {}

trait FromNode: Sized {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self>;
}

trait NodeExt<'a>: Sized {
    fn get_scope(&self) -> Scope;
    fn get_scope_renamed<S: AsRef<str>>(&self, name: S) -> Scope;
    fn validate_tag_name<S: AsRef<str>>(&self, name: S, errors: &mut Vec<PNMLError>) -> bool;
    fn get_attribute<S: AsRef<str>>(&self, name: S) -> Option<&'a str>;
    fn pick_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<&'a str>;
    fn get_or_make_id(&self) -> String;
    fn pick_or_make_id(&self, errors: &mut Vec<PNMLError>) -> String;
    fn parse_float_attribute(&self, spec: &str, errors: &mut Vec<PNMLError>) -> Option<f64>;
    fn get_float_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<f64>;
    fn pick_float_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<f64>;
    fn parse_color_attribute(&self, spec: &str, errors: &mut Vec<PNMLError>) -> Option<Color>;
    fn get_color_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Color>;
    fn pick_color_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Color>;
    fn get_first_element<S: AsRef<str>>(&self, name: S) -> Option<Self>;
    fn pick_first_element<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self>;
    fn get_text_of_first_element<S: AsRef<str>>(&self, name: S) -> Option<&'a str>;
    fn pick_text_of_first_element<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<&'a str>;
}

impl<'a, 'input> NodeExt<'a> for xml::Node<'a, 'input> {
    #[inline]
    fn get_scope(&self) -> Scope {
        self.get_scope_renamed(self.tag_name().name())
    }

    fn get_scope_renamed<S: AsRef<str>>(&self, name: S) -> Scope {
        let doc = self.document();
        let name = name.as_ref().into();
        let byte_range = self.range();
        let text_range = doc.text_pos_at(byte_range.start)..doc.text_pos_at(byte_range.end);

        Scope { name, byte_range, text_range }
    }

    fn validate_tag_name<S: AsRef<str>>(&self, name: S, errors: &mut Vec<PNMLError>) -> bool {
        let name = name.as_ref();

        self.has_tag_name(name) || {
            errors.push(PNMLError::invalid_tag(self, name));

            false
        }
    }

    #[inline]
    fn get_attribute<S: AsRef<str>>(&self, name: S) -> Option<&'a str> {
        self.attribute(name.as_ref())
    }

    fn pick_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<&'a str> {
        let name = name.as_ref();

        self.get_attribute(name).or_else(|| {
            errors.push(PNMLError::MissingAttribute(self.get_scope(), name.into()));

            None
        })
    }

    fn get_or_make_id(&self) -> String {
        self.get_attribute("id").map(Into::into).unwrap_or_else(|| {
            format!("[[anonymous-{}]]", self.document().text_pos_at(self.range().start))
        })
    }

    fn pick_or_make_id(&self, errors: &mut Vec<PNMLError>) -> String {
        self.pick_attribute("id", errors).map(Into::into).unwrap_or_else(|| {
            format!("[[anonymous-{}]]", self.document().text_pos_at(self.range().start))
        })
    }

    fn parse_float_attribute(&self, spec: &str, errors: &mut Vec<PNMLError>) -> Option<f64> {
        spec.parse().map_or_else(
            |err| {
                errors.push(PNMLError::BadFloat(self.get_scope(), err));
                None
            },
            Some,
        )
    }

    #[inline]
    fn get_float_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<f64> {
        self.get_attribute(name).and_then(|spec| self.parse_float_attribute(spec, errors))
    }

    #[inline]
    fn pick_float_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<f64> {
        self.pick_attribute(name, errors).and_then(|spec| self.parse_float_attribute(spec, errors))
    }

    fn parse_color_attribute(&self, spec: &str, errors: &mut Vec<PNMLError>) -> Option<Color> {
        spec.parse().map_or_else(
            |err| {
                errors.push(PNMLError::BadColor(self.get_scope(), err));
                None
            },
            Some,
        )
    }

    #[inline]
    fn get_color_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Color> {
        self.get_attribute(name).and_then(|spec| self.parse_color_attribute(spec, errors))
    }

    #[inline]
    fn pick_color_attribute<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Color> {
        self.pick_attribute(name, errors).and_then(|spec| self.parse_color_attribute(spec, errors))
    }

    #[inline]
    fn get_first_element<S: AsRef<str>>(&self, name: S) -> Option<Self> {
        self.children().find(|n| n.is_element() && n.tag_name().name() == name.as_ref())
    }

    fn pick_first_element<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        let name = name.as_ref();

        self.get_first_element(name).or_else(|| {
            errors.push(PNMLError::MissingElement(self.get_scope(), name.into()));

            None
        })
    }

    #[inline]
    fn get_text_of_first_element<S: AsRef<str>>(&self, name: S) -> Option<&'a str> {
        self.get_first_element(name).and_then(|elt| elt.text())
    }

    fn pick_text_of_first_element<S: AsRef<str>>(
        &self,
        name: S,
        errors: &mut Vec<PNMLError>,
    ) -> Option<&'a str> {
        self.pick_first_element(name, errors).and_then(|elt| {
            elt.text().or_else(|| {
                errors.push(PNMLError::MissingText(self.get_scope(), elt.get_scope()));

                None
            })
        })
    }
}

#[derive(Debug)]
struct Color(css::RGBA);

impl FromStr for Color {
    type Err = String;

    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        let mut input = css::ParserInput::new(spec);
        let mut parser = css::Parser::new(&mut input);

        match css::Color::parse(&mut parser) {
            Ok(color) => match color {
                css::Color::RGBA(rgba) => Ok(Color(rgba)),
                css::Color::CurrentColor => Err("\"currentcolor\" isn't supported".into()),
            },
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}

#[derive(Debug)]
struct Coordinates {
    x: f64,
    y: f64,
}

impl FromNode for Coordinates {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        node.pick_float_attribute("x", errors)
            .and_then(|x| node.pick_float_attribute("y", errors).map(|y| Coordinates { x, y }))
    }
}

#[derive(Debug)]
struct Dimension {
    x: f64,
    y: f64,
}

impl FromNode for Dimension {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        // FIXME is-nonnegative validation
        node.pick_float_attribute("x", errors)
            .and_then(|x| node.pick_float_attribute("y", errors).map(|y| Dimension { x, y }))
    }
}

#[derive(Debug)]
enum GradientRotation {
    Vertical,
    Horizontal,
    Diagonal,
}

impl FromNode for GradientRotation {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        node.get_attribute("gradient-rotation").and_then(|spec| match spec {
            "vertical" => Some(GradientRotation::Vertical),
            "horizontal" => Some(GradientRotation::Horizontal),
            "diagonal" => Some(GradientRotation::Diagonal),
            _ => {
                errors.push(PNMLError::InvalidAttributeValue(
                    node.get_scope(),
                    "gradient rotation".into(),
                    spec.into(),
                ));

                None
            }
        })
    }
}

#[derive(Debug)]
struct Fill {
    color:             Option<Color>,
    image:             Option<String>,
    gradient_color:    Option<Color>,
    gradient_rotation: Option<GradientRotation>,
}

impl FromNode for Fill {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("fill", errors) {
            let color = node.get_color_attribute("color", errors);
            let image = node.get_attribute("image").map(Into::into);
            let gradient_color = node.get_color_attribute("gradient-color", errors);
            let gradient_rotation = GradientRotation::from_node(node, errors);

            Some(Fill { color, image, gradient_color, gradient_rotation })
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum LineShape {
    Line,
    Curve,
}

impl FromNode for LineShape {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        node.pick_attribute("shape", errors).and_then(|spec| match spec {
            "line" => Some(LineShape::Line),
            "curve" => Some(LineShape::Curve),
            _ => {
                errors.push(PNMLError::InvalidAttributeValue(
                    node.get_scope(),
                    "line shape".into(),
                    spec.into(),
                ));

                None
            }
        })
    }
}

#[derive(Debug)]
enum LineStyle {
    Solid,
    Dash,
    Dot,
}

impl FromNode for LineStyle {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        node.get_attribute("style").and_then(|spec| match spec {
            "solid" => Some(LineStyle::Solid),
            "dash" => Some(LineStyle::Dash),
            "dot" => Some(LineStyle::Dot),
            _ => {
                errors.push(PNMLError::InvalidAttributeValue(
                    node.get_scope(),
                    "line style".into(),
                    spec.into(),
                ));

                None
            }
        })
    }
}

#[derive(Debug)]
struct Line {
    shape: Option<LineShape>,
    color: Option<Color>,
    width: Option<f64>,
    style: Option<LineStyle>,
}

impl FromNode for Line {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("line", errors) {
            let shape = LineShape::from_node(node, errors);
            let color = node.get_color_attribute("color", errors);
            // FIXME is-nonnegative validation
            let width = node.get_float_attribute("width", errors);
            let style = LineStyle::from_node(node, errors);

            Some(Line { shape, color, width, style })
        } else {
            None
        }
    }
}

// FIXME attributes
#[derive(Debug)]
struct Font {
    family: Option<String>,
}

impl FromNode for Font {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("font", errors) {
            let family = node.get_attribute("family").map(Into::into);

            Some(Font { family })
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
struct Graphics {
    positions: Vec<Coordinates>,
    dimension: Option<Dimension>,
    offset:    Option<Coordinates>,
    fill:      Option<Fill>,
    line:      Option<Line>,
    font:      Option<Font>,
}

impl FromNode for Graphics {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("graphics", errors) {
            let mut positions = Vec::new();
            let mut dimension = None;
            let mut offset = None;
            let mut fill = None;
            let mut line = None;
            let mut font = None;

            for elt in node.children().filter(|n| n.is_element()) {
                match elt.tag_name().name() {
                    "position" => positions.extend(Coordinates::from_node(elt, errors)),
                    "dimension" => dimension = Dimension::from_node(elt, errors),
                    "offset" => offset = Coordinates::from_node(elt, errors),
                    "fill" => fill = Fill::from_node(elt, errors),
                    "line" => line = Line::from_node(elt, errors),
                    "font" => font = Font::from_node(elt, errors),
                    _ => errors.push(PNMLError::invalid_child_element(&node, &elt)),
                }
            }

            match node.parent_element().map(|elt| elt.tag_name().name()) {
                Some("place") | Some("transition") | Some("page") => if positions.is_empty() {},
                Some("annotation") => {
                    if offset.is_none() {
                        //errors.push(PNMLError::(&node, &elt))
                    }
                }
                Some(_) => {}
                None => {}
            }

            Some(Graphics { positions, dimension, offset, fill, line, font })
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
struct Label {
    text:     Option<String>,
    graphics: Option<Graphics>,
}

impl FromNode for Label {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        let text = node.pick_text_of_first_element("text", errors).map(Into::into);
        let graphics = node
            // FIXME
            .get_first_element("graphics")
            //.pick_first_element("graphics", errors)
            .map(|elt| Graphics::from_node(elt, errors).unwrap_or_else(Graphics::default));

        Some(Label { text, graphics })
    }
}

#[derive(Debug)]
pub struct Place {
    id:              String,
    name:            Option<Label>,
    graphics:        Option<Graphics>,
    initial_marking: Option<Label>,
}

impl Place {
    #[inline]
    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }

    #[inline]
    pub fn get_name_as_text(&self) -> Option<&str> {
        self.name.as_ref().and_then(|label| label.text.as_ref()).map(|s| s.as_str())
    }
}

impl FromNode for Place {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("place", errors) {
            let id = node.pick_or_make_id(errors);
            let mut name = None;
            let mut graphics = None;
            let mut initial_marking = None;

            for elt in node.children().filter(|n| n.is_element()) {
                match elt.tag_name().name() {
                    "name" => name = Label::from_node(elt, errors),
                    "graphics" => graphics = Graphics::from_node(elt, errors),
                    "initialMarking" => initial_marking = Label::from_node(elt, errors),
                    _ => errors.push(PNMLError::invalid_child_element(&node, &elt)),
                }
            }

            Some(Place { id, name, graphics, initial_marking })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Transition {
    id: String,
}

impl Transition {
    #[inline]
    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

impl FromNode for Transition {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("transition", errors) {
            let id = node.pick_or_make_id(errors);

            Some(Transition { id })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Arc {
    id: String,
}

impl Arc {
    #[inline]
    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

impl FromNode for Arc {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("arc", errors) {
            let id = node.get_or_make_id();

            Some(Arc { id })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Page {
    id:          String,
    places:      Vec<Place>,
    transitions: Vec<Transition>,
    arcs:        Vec<Arc>,
}

impl Page {
    #[inline]
    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }

    #[inline]
    pub fn get_places(&self) -> &[Place] {
        self.places.as_slice()
    }

    #[inline]
    pub fn get_transitions(&self) -> &[Transition] {
        self.transitions.as_slice()
    }

    #[inline]
    pub fn get_arcs(&self) -> &[Arc] {
        self.arcs.as_slice()
    }

    pub fn get_place_by_id<S: AsRef<str>>(&self, place_id: S) -> Result<&Place, PetError> {
        let place_id = place_id.as_ref();

        self.places.iter().find(|p| p.get_id() == place_id).ok_or_else(|| {
            PetError::ItemNotFound(PetItemKind::Place, place_id.into(), self.id.clone())
        })
    }

    pub fn get_transition_by_id<S: AsRef<str>>(
        &self,
        transition_id: S,
    ) -> Result<&Transition, PetError> {
        let transition_id = transition_id.as_ref();

        self.transitions.iter().find(|p| p.get_id() == transition_id).ok_or_else(|| {
            PetError::ItemNotFound(PetItemKind::Transition, transition_id.into(), self.id.clone())
        })
    }
}

impl FromNode for Page {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("page", errors) {
            let id = node.pick_or_make_id(errors);
            let mut places = Vec::new();
            let mut transitions = Vec::new();
            let mut arcs = Vec::new();

            for elt in node.children().filter(|n| n.is_element()) {
                match elt.tag_name().name() {
                    "place" => places.extend(Place::from_node(elt, errors)),
                    "transition" => transitions.extend(Transition::from_node(elt, errors)),
                    "arc" => arcs.extend(Arc::from_node(elt, errors)),

                    #[allow(clippy::match_single_binding)]
                    "toolspecific" => match elt.attribute("tool") {
                        // FIXME handle known tools
                        _ => errors.push(PNMLError::unknown_or_missing_tool(&node, &elt)),
                    },

                    _ => errors.push(PNMLError::invalid_child_element(&node, &elt)),
                }
            }

            Some(Page { id, places, transitions, arcs })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Net {
    id:    String,
    kind:  Option<String>,
    name:  Option<String>,
    pages: Vec<Page>,
}

impl Net {
    #[inline]
    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }

    #[inline]
    pub fn get_pages(&self) -> &[Page] {
        self.pages.as_slice()
    }

    pub fn get_page_by_id<S: AsRef<str>>(&self, page_id: S) -> Result<&Page, PetError> {
        let page_id = page_id.as_ref();

        self.pages.iter().find(|p| p.get_id() == page_id).ok_or_else(|| {
            PetError::ItemNotFound(PetItemKind::Page, page_id.into(), self.id.clone())
        })
    }
}

impl FromNode for Net {
    fn from_node<'a, 'input>(
        node: xml::Node<'a, 'input>,
        errors: &mut Vec<PNMLError>,
    ) -> Option<Self> {
        if node.validate_tag_name("net", errors) {
            let id = node.pick_or_make_id(errors);
            let kind = node.pick_attribute("type", errors).map(Into::into);
            let mut name = None;
            let mut pages = Vec::new();

            for elt in node.children().filter(|n| n.is_element()) {
                match elt.tag_name().name() {
                    "name" => name = elt.pick_text_of_first_element("text", errors).map(Into::into),
                    "page" => pages.extend(Page::from_node(elt, errors)),
                    _ => errors.push(PNMLError::invalid_child_element(&node, &elt)),
                }
            }

            Some(Net { id, kind, name, pages })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PNML {
    path:      Option<PathBuf>,
    namespace: Option<String>,
    nets:      Vec<Net>,
    errors:    Vec<PNMLError>,
}

impl PNML {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PetError> {
        let path_buf = path.as_ref().into();
        let spec = fs::read_to_string(path)?;
        let mut pnml: Result<Self, PetError> = spec.parse().map_err(Into::into);

        if let Ok(ref mut pnml) = pnml {
            pnml.path = Some(path_buf);
        }

        pnml
    }

    pub fn get_nets(&self) -> &[Net] {
        self.nets.as_slice()
    }

    pub fn get_net_by_id<S: AsRef<str>>(&self, net_id: S) -> Result<&Net, PetError> {
        let net_id = net_id.as_ref();

        self.nets.iter().find(|n| n.get_id() == net_id).ok_or_else(|| {
            PetError::ItemNotFound(
                PetItemKind::Net,
                net_id.into(),
                self.path.as_ref().map_or_else(String::new, |p| p.display().to_string()),
            )
        })
    }

    pub fn get_errors(&self) -> &[PNMLError] {
        self.errors.as_slice()
    }
}

impl FromStr for PNML {
    type Err = PNMLError;

    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        let path = None;
        let document = xml::Document::parse(spec)?;
        let root_elt = document.root_element();
        let mut errors = Vec::new();

        if root_elt.validate_tag_name("pnml", &mut errors) {
            let namespace = root_elt.default_namespace().map(Into::into);
            let mut nets = Vec::new();

            for net_elt in root_elt.children().filter(|n| n.is_element()) {
                nets.extend(Net::from_node(net_elt, &mut errors));
            }

            Ok(PNML { path, namespace, nets, errors })
        } else if let Some(err) = errors.pop() {
            Err(err)
        } else {
            unreachable!()
        }
    }
}
