
extern crate piston_window;
extern crate sprite;
extern crate fmod_sys;
extern crate winit;
extern crate serde_json;

use piston_window::*;
use sprite::*;
use serde_json::{Result, Value};
use fmod_sys::*;



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


#[allow(dead_code)]
fn print_type_of<T>(_: &T, text: &str) {
  println!("{:?} {:?}", text, std::any::type_name::<T>())
}





fn main() {
  let kauhanen_version = 2.001f32;
  let files = find_files(".");

  // que?
  std::env::set_var("LD_LIBRARY_PATH", ".");

  let music : Option<&std::string::String> = files.iter().find(|s| s.to_lowercase().ends_with(".mp3") || s.to_lowercase().ends_with(".ogg") );

  // fmod init. damn this is ugly
  let mut system: *mut FMOD_SYSTEM = std::ptr::null_mut();
  let result = unsafe { FMOD_System_Create(&mut system) };
  assert_eq!(result, FMOD_RESULT::FMOD_OK);
  let result_init = unsafe{ FMOD_System_Init(system,6,FMOD_DEFAULT,std::ptr::null_mut()) };
  assert_eq!(result_init, FMOD_RESULT::FMOD_OK);
  let mut sound: *mut FMOD_SOUND = std::ptr::null_mut();
  let result_stream = unsafe { FMOD_System_CreateStream(system, music.unwrap().as_ptr() as *const std::os::raw::c_char, FMOD_DEFAULT, std::ptr::null_mut(), &mut sound) };
  assert_eq!(result_stream, FMOD_RESULT::FMOD_OK);
  let mut music_length :u32 = 0;
  unsafe { FMOD_Sound_GetLength( sound, &mut music_length, FMOD_TIMEUNIT_MS ) };


  // match found filenames to ones in json, if exists, otherwise take all images
  let images : Vec<&std::string::String> = match files.iter().find( |s| s.to_lowercase().ends_with(".json") && !s.contains("debug") && !s.contains("/.") ) {
    Some(filename) => {
      let file = std::fs::File::open(filename);
      let reader = std::io::BufReader::new(file.unwrap());
      let u : Result<serde_json::Value> = serde_json::from_reader(reader);
      u.unwrap().as_array().unwrap().iter().map( |s| {
          if s.is_string() {
            return s.as_str();
          }
          else if s.is_object() {
            return s["filename"].as_str();
          }
          return None;
        }
      )
      .filter( |s| s.is_some() )
      .map( |s|
        files.iter().find(|k| k.ends_with(s.unwrap()) ).unwrap()
      )
      .collect::<Vec<_>>()
    }
    None => {
      files.iter().filter(|s|
        s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png")
      ).collect()
    }
  };

  //println!("{:?}", images);
  //std::process::exit(1);


  // fuck it. use winit to get screen resolution as piston sucks
  let events_loop = winit::event_loop::EventLoop::new();
  // might be get_current_monitor() but no clue which one piston will use
  let monitor = events_loop.primary_monitor().unwrap();
  let monitor_size = monitor.size();
  let screen_width = monitor_size.width;
  let screen_height = monitor_size.height;

  let opengl = OpenGL::V3_2;
  let mut window: PistonWindow = WindowSettings::new( format!("kauhanen {}", kauhanen_version), [screen_width,screen_height] )
      .exit_on_esc(true)
      .graphics_api(opengl)
      .fullscreen(false)
      .build()
      .unwrap();


  let mut texture_context = TextureContext {
    factory: window.factory.clone(),
    encoder: window.factory.create_command_buffer().into()
  };


    // load images  and create textures
  let mut images_loaded : Vec<sprite::Sprite<_>> =
    images.iter().map(|s|
      std::rc::Rc::new(Texture::from_path(
        &mut texture_context,
        s,
        Flip::None,
        &TextureSettings::new()
      ).unwrap())
    )
    .map(|s|
      Sprite::from_texture(s.clone())
    )
    .collect();


  println!("wifth: {:?} {:?}", screen_height, screen_width);

  // center and scale sprites
  //let screen_aspect = (screen_width as f64) / (screen_height as f64);
  for s in &mut images_loaded {
    let (spr_x,spr_y) = s.get_texture().get_size();
    let scale_y = (screen_height as f64) / (spr_y as f64);
    let scale_x = (screen_width as f64) / (spr_x as f64);
    s.set_scale( scale_x, scale_y );
    s.set_position( screen_width as f64/2.0 as f64, screen_height as f64/2.0 as f64 );
  }

  let music_posses : Vec<u32> = (0..images.len()).map(|s| (s as u32 *music_length/images.len() as u32 ) ).collect();



  let mut last = 0;
  let mut spr = None;
  let mut image_iter = images_loaded.iter();


  let mut channel: *mut FMOD_CHANNEL = std::ptr::null_mut();
  let result_play = unsafe { FMOD_System_PlaySound(system, sound, std::ptr::null_mut(), 0, &mut channel ) };
  assert_eq!(result_play, FMOD_RESULT::FMOD_OK);


  while let Some(e) = window.next() {
    let mut pos: u32 = 0;
    unsafe { FMOD_Channel_GetPosition( channel, &mut pos, FMOD_TIMEUNIT_MS) };
    if last < music_posses.len() && pos >= music_posses[last]
    {
      spr = image_iter.next();
      last = last + 1;
    }

    window.draw_2d(&e, |c, g, _| {
        clear([1.0, 1.0, 1.0, 1.0], g);
        spr.unwrap().draw(c.transform, g);
    });
    let mut is_playing: FMOD_BOOL = 0;
    unsafe { FMOD_Channel_IsPlaying( channel, &mut is_playing ) };
    if is_playing == 0
    {
      return;
    }
  }

  let result_sound_release = unsafe { FMOD_Sound_Release( sound ) };
  assert_eq!(result_sound_release, FMOD_RESULT::FMOD_OK);
  let result_system_release = unsafe { FMOD_System_Release( system ) };
  assert_eq!(result_system_release, FMOD_RESULT::FMOD_OK);

}

