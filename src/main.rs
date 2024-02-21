use std::{io::Write, thread, time::{Duration, Instant}};
use crossterm::{event::{poll, read, Event, KeyCode}, terminal::ClearType, QueueableCommand};
use rand::{thread_rng, Rng};
const SCREEN_WIDTH: isize = 50;
const SCREEN_HEIGHT: isize = 30;

#[derive(PartialEq)]
enum Edge {
    Ceiling,
    Floor 
}

struct  Block {
    pixel: Pixel,
    x_float: f64,
}

struct Wall {
    blocks: Vec<Block>,
    edge_touching: Edge,
    width: isize,
    height: isize,
    velocity: f64,
    offscreen: bool
}

impl Wall {
    fn new(vel: f64) -> Self {
        let mut edge_touching = Edge::Ceiling;
        if rand::random() {
            edge_touching = Edge::Floor;
        }

        let mut w = Wall{
            blocks: vec![],
            edge_touching,
            width: 2,
            height: 2 * SCREEN_HEIGHT / 3,
            velocity: vel,
            offscreen: false
        };
        let mut rng = thread_rng();
        w.height -= rng.gen_range(0..(SCREEN_HEIGHT/4));
        for i in 0..w.width {
            let x = SCREEN_WIDTH - i;     
                for j in 1..w.height {
                    match w.edge_touching {
                        Edge::Floor => {
                            let y = SCREEN_HEIGHT - j;
                            w.blocks.push(
                                Block {
                                    pixel: Pixel{x, y},
                                    x_float: x as f64,
                                }
                            );
                        },
                        Edge::Ceiling => {
                            let y = j - 1;
                            w.blocks.push(
                                Block {
                                    pixel: Pixel{x, y},
                                    x_float: x as f64,
                                }
                            );
                        }
                    }
                }
            }
        w
    }
    fn update_pixel(&mut self) {
        let mut offscreen = true;
        for block in self.blocks.iter_mut() {
            block.x_float -= self.velocity;
            block.pixel.x = block.x_float as isize;
            if block.pixel.x >= 0 {
                offscreen = false;
            }
        }
        self.offscreen = offscreen;

    }
}

struct Player {
    pixel: Pixel,
    height: f64,
    velocity: f64,
    score: f64
}

impl Player {
    fn new() -> Self {
        Player{
            pixel: Pixel { x: 1, y: SCREEN_HEIGHT / 2 },
            height: SCREEN_HEIGHT as f64 / 2.0,
            velocity: 0.0,
            score: 0.0
        }
    }
    fn flap(&mut self) {
        self.velocity = -0.5 ;
    }
    fn height_to_pixel(&mut self) {
        self.pixel.y = self.height as isize
    }
}

#[derive(PartialEq, Debug)]
struct Pixel {
    x: isize,
    y: isize
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Tile {
    Sky,
    Bird,
    Wall
}

enum Command {
    Quit,
    Flap,
    // Pause
}

struct Screen {
    pixels: Vec<Vec<Tile>>
}

impl Screen {
    fn new() -> Self {
        let mut s = Screen{pixels: vec![]};
        for _ in 0..SCREEN_HEIGHT{
            let mut v2: Vec<Tile> = vec![];
            for _ in 0..SCREEN_WIDTH {
                v2.push(Tile::Sky);
            }
            s.pixels.push(v2);
        }
        s
    }
    fn set(&mut self, pixel: &Pixel, tile: Tile) {
        if pixel.y < SCREEN_HEIGHT && pixel.y >= 0 && pixel.x < SCREEN_WIDTH && pixel.x >= 0 {
            self.pixels[pixel.y as usize][pixel.x as usize] = tile;
        }
    }
    fn refresh(&mut self) {
        for i in 0..self.pixels.len(){
            for j in 0..self.pixels[0].len() {
                self.pixels[i][j] = Tile::Sky;
            }
        }
    }

    fn display(&self) {
        for row in self.pixels.iter() {
            for tile in row.iter() {
                match tile {
                    Tile::Sky => {
                        print!("  ")
                    },
                    Tile::Bird=> {   
                        print!("B ")
                    },
                    Tile::Wall => {
                        print!("# ")
                    }
                }
            }
            print!("\n\r");            
        }
    }
}

fn clear_screen() {
    let mut out = std::io::stdout();
    out.queue(crossterm::terminal::Clear(ClearType::All)).unwrap();
    out.queue(crossterm::cursor::Hide).unwrap();
    out.queue(crossterm::cursor::MoveTo(0,0)).unwrap();
    out.flush().unwrap();
}

fn keypress(dur: Duration) -> Option<Command> {
    if poll(dur).unwrap() {
        match read().unwrap() {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Char('q') => {
                        return Some(Command::Quit);
                    },
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char(' ') => {
                        return Some(Command::Flap);
                    },
                    _ => None
                }
            },
            _ => None
        }
    } else {
        None
    }

}

fn run_game() {
    clear_screen();

    let wall_velocity = 0.55/2.0;

    let mut walls: Vec<Wall> = vec![];

    let target_fps = 60;
    let tick_rate = 1000u64/target_fps;
    let tick_duration = Duration::from_millis(tick_rate);

    let grav: f64 = 1.0;
    let grav_per_frame = grav / target_fps as f64;

    let mut player = Player::new();
    let mut screen = Screen::new();

    let mut tick_counter = 0u64;
    let mut new_wall_counter = 60u64;
    let mut rng = thread_rng();



    'game: loop {
        let now = Instant::now();
        

        let key: Option<Command> = keypress(tick_duration);
        match key {
            Some(Command::Flap) => player.flap(),
            Some(Command::Quit) => std::process::exit(0),
            None => {}
        }

        player.velocity += grav_per_frame;
        player.height += player.velocity;

        walls.retain(|wall| wall.offscreen == false);

        if tick_counter % new_wall_counter == 0 {
            walls.push(Wall::new(wall_velocity));
            new_wall_counter = rng.gen_range(50..75);
            tick_counter = 0;

        }


        screen.refresh();
        player.height_to_pixel();


        for wall in walls.iter_mut() {
            wall.update_pixel();
            for block in wall.blocks.iter() {
                screen.set(&block.pixel, Tile::Wall)
            }
        }

        screen.set(&player.pixel, Tile::Bird);


        clear_screen();
        screen.display();

        player.score += wall_velocity;

        tick_counter += 1;

        let elapsed_time = now.elapsed();
        if elapsed_time < tick_duration {
            thread::sleep(tick_duration - elapsed_time)
        }
        print!("{:#?}FPS\n\r", 1000 / now.elapsed().as_millis());
        print!("Score: {}\r\n", player.score as isize);

        if player.pixel.y < 0 || player.pixel.y >= SCREEN_HEIGHT {
            break 'game;
        }
        for wall in walls.iter() {
            for block in wall.blocks.iter() {
                if block.pixel == player.pixel {
                    break 'game;
                }
            }
        }
    }

}

fn main() {
    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        run_game();
        thread::sleep(Duration::from_millis(3000));
    }
}
