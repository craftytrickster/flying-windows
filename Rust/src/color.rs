#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    YellowishBronze, // made up name
    Teal,
    Aqua,
    Lime,
    Purple,
    Gray,
    White,
    Red,
    Maroon,
    Green,
    Blue,
}

pub const ALL_COLORS: [Color; 11] = [
    Color::YellowishBronze,
    Color::Teal,
    Color::Aqua,
    Color::Lime,
    Color::Purple,
    Color::Gray,
    Color::White,
    Color::Red,
    Color::Maroon,
    Color::Green,
    Color::Blue,
];

impl Color {
    pub fn as_web_color(&self) -> &'static str {
        match self {
            Color::YellowishBronze => "#666c2b",
            Color::Teal => "teal",
            Color::Aqua => "aqua",
            Color::Lime => "lime",
            Color::Purple => "purple",
            Color::Gray => "gray",
            Color::White => "white",
            Color::Red => "red",
            Color::Maroon => "maroon",
            Color::Green => "green",
            Color::Blue => "blue",
        }
    }
}
