

// find fmodex64_vc.lib and fmodex64.dll somewhere
// for example https://github.com/daisukewi/DamnedSunset/tree/master/dependencies/bin/Release


extern crate piston_window;
extern crate sprite;

extern crate rfmod;
extern crate winit;

use winit::{KeyboardInput, VirtualKeyCode, WindowEvent};


use piston_window::*;
use sprite::*;

#[link(name = "fmodex64_vc.ilb")]

fn find_files(dir: &str) -> Vec<std::string::String> 
{
    let paths = std::fs::read_dir(dir).unwrap();
    let mut images : Vec<std::string::String> = vec![];
    for path in paths {
        let p = path.unwrap();
        if p.file_type().unwrap().is_dir()
        {
            images.append( &mut find_files( p.path().to_str().unwrap() ) );
        }
        else
        {
            images.push( p.path().into_os_string().into_string().unwrap() );
        }
    }
    images
}


fn init_fmod( filename:&str ) -> rfmod::Sound
{
    let fmod = match rfmod::Sys::new() {
        Ok(f) => f,
        Err(e) => {
            panic!("Error code : {:?}", e);
        }
    };
    match fmod.init() {
        rfmod::Status::Ok => {}
        e => {
            panic!("FmodSys.init failed : {:?}", e);
        }
    };

    let sound = match fmod.create_sound(filename, None, None) {
        Ok(s) => s,
        Err(err) => {
            panic!("Error code : {:?}", err);
        }
    };
    sound
}


fn main() {
    let files = find_files(".");
    let music : Option<&std::string::String> = files.iter().find(|s| 
        s.to_lowercase().ends_with(".mp3") || s.to_lowercase().ends_with(".ogg") );
    let images : Vec<&std::string::String> = files.iter().filter(|s| 
        s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png") 
        ).collect();

    std::env::set_var("LD_LIBRARY_PATH", ".");

    if music == None
    {
        panic!("ei musaa hv kaikki");
    }


    let fmod = match rfmod::Sys::new() {
        Ok(f) => f,
        Err(e) => {
            panic!("Error code : {:?}", e);
        }
    };
    match fmod.init() {
        rfmod::Status::Ok => {}
        e => {
            panic!("FmodSys.init failed : {:?}", e);
        }
    };

    let sound = match fmod.create_sound(music.unwrap(), None, None) {
        Ok(s) => s,
        Err(err) => {
            panic!("Error code : {:?}", err);
        }
    };

    let music_length = sound.get_length( rfmod::TIMEUNIT_MS );


    // fuck it. use winit to get screen resolution as piston sucks
    let events_loop = winit::EventsLoop::new();
    // might be get_current_monitor() but no clue which one piston will use
    let monitor: winit::MonitorId = events_loop.get_primary_monitor();
    let (screen_width, screen_height) = monitor.get_dimensions();

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("kauhanen", [screen_width,screen_height])
        .exit_on_esc(true)
        .opengl(opengl)
        .fullscreen(true)
        .build()
        .unwrap();

    // load images  and create textures
    let mut images_loaded : Vec<sprite::Sprite<_>> = 
        images.iter().map(|s| 
            std::rc::Rc::new(Texture::from_path(
                &mut window.factory,
                s,
                Flip::None,
                &TextureSettings::new()
            ).unwrap())
        )
        .map(|s| 
            Sprite::from_texture(s.clone())
        )
        .collect();


    // center and scale sprites
    let screen_aspect = (screen_width as f64) / (screen_height as f64);
    for s in &mut images_loaded {
        let bb = s.bounding_box();
        let (spr_x,spr_y) = s.get_texture().get_size();
        let scale_y = (screen_height as f64) / (spr_y as f64);
        let scale_x = (screen_width as f64) / (spr_x as f64);
        s.set_scale( scale_x, scale_y );
        s.set_position( screen_width as f64/2.0 as f64, screen_height as f64/2.0 as f64 );
        println!("{:?}", s.get_texture().get_size());
    }


    let music_posses : Vec<usize> = (0..images.len()).map(|s| (s*music_length.unwrap() as usize/images.len()) ).collect();


    
    let mut last = 0;


    let mut spr = None;
    let mut image_iter = images_loaded.iter();
    let channel = sound.play().unwrap();
    while let Some(e) = window.next() {
        let pos = channel.get_position(rfmod::TIMEUNIT_MS);
        if ( last < music_posses.len() && pos.unwrap() >= music_posses[last] )
        {
            spr = image_iter.next();
            last = last + 1;
        }

        window.draw_2d(&e, |c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            spr.unwrap().draw(c.transform, g);
        });
        if channel.is_playing() == Ok(false)
        {
            return;
        }
    }
 

}

