

// find fmodex64_vc.lib and fmodex64.dll somewhere
// for example https://github.com/daisukewi/DamnedSunset/tree/master/dependencies/bin/Release


extern crate piston_window;
extern crate sprite;
extern crate rfmod;
extern crate winit;
extern crate serde_json;

use piston_window::*;
use sprite::*;
use serde_json::{Result, Value};


fn find_files(dir: &str) -> Vec<std::string::String>
{
    let paths = std::fs::read_dir(dir).unwrap();
    let mut files : Vec<std::string::String> = vec![];
    for path in paths {
        let p = path.unwrap();
        if p.file_type().unwrap().is_dir()
        {
            files.append( &mut find_files( p.path().to_str().unwrap() ) );
        }
        else
        {
            files.push( p.path().into_os_string().into_string().unwrap() );
        }
    }
    files
}



fn init_fmod() -> rfmod::Sys
{
    let fmod = match rfmod::Sys::new() {
        Ok(f) => {
            f.init_with_parameters(10i32, rfmod::InitFlag(rfmod::INIT_NORMAL));
            f
        }
        Err(e) => panic!("FmodSys.new : {:?}", e),
    };
    fmod
}




fn main() {
    let files = find_files(".");

    std::env::set_var("LD_LIBRARY_PATH", ".");


    let music : Option<&std::string::String> = files.iter().find(|s|
        s.to_lowercase().ends_with(".mp3") || s.to_lowercase().ends_with(".ogg") );

    if music == None
    {
        panic!("ei musaa hv kaikki");
    }

    let fmod = init_fmod();
    let sound = match fmod.create_sound(music.unwrap(), None, None) {
        Ok(s) => s,
        Err(err) => {
            panic!("Error code : {:?}", err);
        }
    };

    let music_length = sound.get_length( rfmod::TIMEUNIT_MS );



    let images : Vec<&std::string::String> = match files.iter().find( |s| s.to_lowercase().ends_with(".json") && !s.contains("debug") && !s.contains("/.") ) {
        Some(filename) => {
            let file = std::fs::File::open(filename);
            let reader = std::io::BufReader::new(file.unwrap());
            let u : Result<serde_json::Value> = serde_json::from_reader(reader);
            u.unwrap().as_array().unwrap().iter().map( |s| files.iter().find(|k| k.ends_with(s.as_str().unwrap())).unwrap() ).collect()
        }
        None => {
            files.iter().filter(|s|
                s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png")
            ).collect()
        }
    };



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
        let (spr_x,spr_y) = s.get_texture().get_size();
        let scale_y = (screen_height as f64) / (spr_y as f64);
        let scale_x = (screen_width as f64) / (spr_x as f64);
        s.set_scale( scale_x, scale_y );
        s.set_position( screen_width as f64/2.0 as f64, screen_height as f64/2.0 as f64 );
    }


    let music_posses : Vec<usize> = (0..images.len()).map(|s| (s*music_length.unwrap() as usize/images.len()) ).collect();



    let mut last = 0;


    let mut spr = None;
    let mut image_iter = images_loaded.iter();


    let channel = match sound.play() {
        Ok(s) => s,
        Err(err) => {
            panic!("Kauhanen Error code : {:?}", err);
        }
    };

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

