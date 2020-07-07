extern crate ncurses;

use ncurses::*;
use std::{thread, time};
use rand::Rng;

const TIME_INC: u64 = 20;
const SUBPIXEL: i32 = 1000;
const GRAVITY: i32 = 100;
const INERT_WIND: i32 = 50;
const INERT_GRAV: i32 = 20;
const INERT_AIR_FRICTION: i32 = 80;

enum BlastType {
	Inert,
	Normal,
  //Shaped,
}

struct Pellet {
	pos: (i32, i32),
	vel: (i32, i32),
	blast_type: BlastType,
}

impl Pellet {
	fn advance(&mut self) {
		match self.blast_type {
			BlastType::Inert => {
        // These inert pellets slow quickly and hang in the air
				self.pos = (self.pos.0 + self.vel.0, self.pos.1 + self.vel.1);
				self.vel = (
          (self.vel.0+INERT_WIND) * INERT_AIR_FRICTION / 100, 
          (self.vel.1+INERT_GRAV) * INERT_AIR_FRICTION / 100
        );
				mvprintw(self.pos.1/SUBPIXEL, self.pos.0/SUBPIXEL, ".");
			}
      _ => {
        // All other pellets follow parabolic paths
        self.pos = (self.pos.0 + self.vel.0, self.pos.1 + self.vel.1);
        self.vel = (self.vel.0, self.vel.1+GRAVITY);
        mvprintw(self.pos.1/SUBPIXEL, self.pos.0/SUBPIXEL, "o");
      }
		}
	}
}

fn main()
{
  /* Start ncurses. */
  initscr();
  keypad(stdscr(), true);
  noecho();
  timeout(30);
  /* Get the screen bounds. */
  let mut max_x = 0;
  let mut max_y = 0;
  getmaxyx(stdscr(), &mut max_y, &mut max_x);

  let mut pellets: Vec<Pellet> = Vec::new();
  let mut exploders: Vec<usize> = Vec::new();
  let mut vanishers: Vec<usize> = Vec::new();

  let mut ch = getch();

  box_(stdscr(),0,0);

  while ch != 96 // tilde to quit
  {
  	clear();
  	for p in 0..pellets.len() {
  		pellets[p].advance();
  		if pellets[p].vel.1 > 0 && !matches!(pellets[p].blast_type, BlastType::Inert) {
        // at zenith, active fireworks explode and vanish
  			exploders.push(p);
  			vanishers.push(p);
  		} else if pellets[p].pos.1 > max_y*SUBPIXEL {
        // any pellets that go below bottom of window vanish
  			vanishers.push(p);
  		}
  	}
  	for p in &exploders {
      // radiating pellets from blast are equally spaced by angle
  		let num_radial = rand::thread_rng().gen_range(10,16);
      let magnitude = rand::thread_rng().gen_range(1000,4000) as f64;
      let coeff = std::f64::consts::PI * 2.0 / (num_radial as f64);
  		for r in 0..num_radial {
  			let mag_x = (magnitude * (r as f64 * coeff).sin()) as i32;
  			let mag_y = (magnitude * 0.7 * (r as f64 * coeff).cos()) as i32;
	  		pellets.push( Pellet {
	  			pos: (pellets[*p].pos.0, pellets[*p].pos.1),
	  			vel: (mag_x,mag_y),
	  			blast_type: BlastType::Inert,
	  		});
  		}
      // second, smaller ring of pellets
      for r in 0..num_radial/2 {
        let mag_x = (magnitude * 0.5 * (r as f64 * coeff * 2.0).sin()) as i32;
        let mag_y = (magnitude * 0.35 * (r as f64 * coeff * 2.0).cos()) as i32;
        pellets.push( Pellet {
          pos: (pellets[*p].pos.0, pellets[*p].pos.1),
          vel: (mag_x,mag_y),
          blast_type: BlastType::Inert,
        });
      }
  	}
    // vanishers must be removed in decreasing order to avoid errors
  	vanishers.sort_unstable();
  	vanishers.reverse();
  	for p in &vanishers {
  		pellets.swap_remove(*p);
  	}
  	exploders.clear();
  	vanishers.clear();
  	//mvprintw(0, 0, &ch.to_string());
  	thread::sleep(time::Duration::from_millis(TIME_INC));
  	ch = getch(); 
  	if ch >= 49 && ch <= 57 {
      // upper_lim is the initial vertical velocity required for a pellet
      // to nearly touch the top of the screen
      let upper_lim = (((2000 * GRAVITY * (max_y - 3)) as f64).sqrt() as i32) * -1;
      let vel_x = rand::thread_rng().gen_range(-1000,1000);
      let vel_y = rand::thread_rng().gen_range(upper_lim, -1500);
  		pellets.push( Pellet {
		  	pos: (SUBPIXEL * (max_x * (ch-48) / 10), max_y*SUBPIXEL),
		  	vel: (vel_x, vel_y),
		  	blast_type: BlastType::Normal,
  		});
  	}
  }

  /* Terminate ncurses. */
  mv(max_y -1, 0);
  endwin();
}