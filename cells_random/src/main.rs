#![allow(unused_assignments)]
//-------------------------------------------------------------------------
//  Maze generator based in the algorithm of wikipedia:
//  https://en.wikipedia.org/wiki/Maze_generation_algorithm
//-------------------------------------------------------------------------
//-------------------------------------------------------------------------
//                        includes
//-------------------------------------------------------------------------
extern crate sfml;
use sfml::graphics::{CircleShape, Color, Font, RectangleShape, RenderTarget, RenderWindow, Shape,
                     Text, Transformable};
use sfml::system::{Clock, Time, Vector2f, Vector2u};
use sfml::window::{ContextSettings, Event, Key, Style};
//-------------------------------------------------------------------------
//                        globals
//-------------------------------------------------------------------------
const WIDTH: u32 = 400; // is rectangular grid
const HEIGHT: u32 = 400; // is rectangular grid
const W:      u32 = 40;

static COLS: u32 = WIDTH / W;
static ROWS: u32 = HEIGHT / W;


// cell

#[derive(Debug, Clone)]
struct Cell<'a> {
    row: u32,
    column: u32,
    shape: RectangleShape<'a>,
    color: Color,
}

impl<'a> Cell<'a> {
    // constructor
    fn new(row: u32, column: u32, c: Color) -> Self {
        let mut s = RectangleShape::new();
        // NOTE(elsuizo:2018-09-06): le agrego el offset de su ancho
        let origin = Vector2f::new((row*W) as f32, (column*W) as f32);
        s.set_size(Vector2f::new(W as f32, W as f32));
        s.set_outline_thickness(2.0); // no se si va
        s.set_fill_color(&c);
        s.set_outline_color(&Color::BLACK);
        s.set_origin(Vector2f{x: (row / 2) as f32, y: (column / 2) as f32});
        s.set_position(origin);
        Cell {
            row: row,
            column: column,
            shape: s,
            color: c,
        }
    }

    // fn show(mut ) {
    //     let x = self.row * W;
    //     let y = self.column * W;
    // }
}

// NOTE(elsuizo:2018-09-06): quiero elegir aleatoriamente el color de las cells
//-------------------------------------------------------------------------
//                      random
//-------------------------------------------------------------------------
extern crate rand;
use rand::{Rng};

fn pick_random_color() -> Color {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0, 4) {
        0 => Color::RED,
        1 => Color::GREEN,
        2 => Color::WHITE,
        _ => Color::BLUE,
    }
}

fn main() {
    // optional
    let aa_level = 0;
    //-------------------------------------------------------------------------
    //                        window creation
    //-------------------------------------------------------------------------
    let context_settings = ContextSettings {
        antialiasing_level: aa_level,
        ..Default::default()
    };
    let mut window = RenderWindow::new(
        (WIDTH, HEIGHT),
        "Maze generator",
        Style::CLOSE,
        &context_settings,
    );

    window.set_vertical_sync_enabled(true);

    let cells_numbers = COLS as usize * ROWS as usize;
    let mut grid: Vec<Cell> = Vec::new();
    for i in 0..COLS {
        for j in 0..ROWS {
            let c: Color = pick_random_color();
            println!("c{:?}", c);
            grid.push(Cell::new(i, j, c));
        }
    }
    println!("COLS is: {:}", COLS as f32);
    println!("ROWS is: {:}", ROWS);
    println!("cells number: {:}", grid.len());
    println!("lalal: {:?}", grid[3].column);
    //-------------------------------------------------------------------------
    //                        loop principal
    //-------------------------------------------------------------------------
    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => return,
                _ => {}
            }
        }

        for i in 0..COLS {
            for j in 0..ROWS {
                let c: Color = pick_random_color();
                grid.push(Cell::new(i, j, c));
        }
    }
        window.clear(&Color::BLACK);
        for cell in &grid {
            window.draw(&cell.shape);
        }
        window.display();
    }
}
