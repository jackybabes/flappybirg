use std::{io::Write, thread, time::{Duration, Instant}};
use crossterm::{event::{poll, read, Event, KeyCode}, terminal::ClearType, QueueableCommand};
// use rand::{thread_rng, Rng};
const SCREEN_WIDTH: usize = 50;
const SCREEN_HEIGHT: usize = 30;


struct Player {
    pixel: Pixel,
    height: f64,
    velocity: f64
}

impl Player {
    fn new() -> Self {
        Player{
            pixel: Pixel { x: 1, y: SCREEN_HEIGHT / 2 },
            height: SCREEN_HEIGHT as f64 / 2.0,
            velocity: 0.0
        }
    }
    fn flap(&mut self) {
        self.velocity = -1.0 ;
    }
    fn height_to_pixel(&mut self) {
        if self.height as usize >= 30 {
            self.pixel.y = 29
        } else {
        self.pixel.y = self.height as usize
        }
    }
}

#[derive(PartialEq)]
struct Pixel {
    x: usize,
    y: usize
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Tile {
    Sky,
    Bird
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
        self.pixels[pixel.y][pixel.x] = tile;
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
                        print!("~ ")
                    },
                    Tile::Bird=> {
                        print!("B ")
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

fn main() {
    crossterm::terminal::enable_raw_mode().unwrap();
    clear_screen();


    let target_fps = 20;
    let tick_rate = 1000u64/target_fps;
    let tick_duration = Duration::from_millis(tick_rate);

    let grav: f64 = 1.5;
    let grav_per_frame = grav / target_fps as f64;

    let mut player = Player::new();
    let mut screen = Screen::new();

    loop {
        let now = Instant::now();
        

        let key: Option<Command> = keypress(tick_duration);
        match key {
            Some(Command::Flap) => player.flap(),
            Some(Command::Quit) => break,
            None => {}
        }



        player.velocity += grav_per_frame;
        player.height += player.velocity;


        screen.refresh();

        player.height_to_pixel();

        screen.set(&player.pixel, Tile::Bird);
        
        clear_screen();
        screen.display();


        let elapsed_time = now.elapsed();
        if elapsed_time < tick_duration {
            thread::sleep(tick_duration - elapsed_time)
        }

        print!("{:#?}FPS", 1000 / now.elapsed().as_millis());
        print!("\n\r")
    }
}
