extern crate piston_window;
extern crate sprite;
extern crate rodio;
extern crate winit;
extern crate serde_json;

use piston_window::*;
use sprite::*;
use serde_json::{Result, Value};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use std::path::Path;


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
  files.sort();
  files
}


fn main() {
    let files = find_files(".");
    
    let music_file = files.iter()
        .find(|s| s.to_lowercase().ends_with(".mp3") || s.to_lowercase().ends_with(".wav"))
        .expect("No music file found");
    
    println!("Found music file: {}", music_file);
    
    let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to get audio output stream");
    let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");

    // Match found filenames to ones in json, if exists, otherwise take all images
    let mut images: Vec<String> = match files.iter().find(|s| s.to_lowercase().ends_with(".json") && !s.contains("debug") && !s.contains("/.")) {
        Some(filename) => {
            println!("Found JSON file: {}", filename);
            match std::fs::File::open(filename) {
                Ok(file) => {
                    let reader = std::io::BufReader::new(file);
                    match serde_json::from_reader::<_, Value>(reader) {
                        Ok(json) => {
                            if let Some(array) = json.as_array() {
                                array.iter()
                                    .filter_map(|s| {
                                        let file_name = if s.is_string() {
                                            s.as_str()
                                        } else if s.is_object() && s.get("filename").is_some() {
                                            s.get("filename").and_then(|f| f.as_str())
                                        } else {
                                            None
                                        };
                                        
                                        file_name.and_then(|name| {
                                            files.iter()
                                                .find(|k| k.ends_with(name))
                                                .map(|s| s.clone())
                                        })
                                    })
                                    .collect()
                            } else {
                                println!("JSON is not an array, falling back to all images");
                                files.iter()
                                    .filter(|s| s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png"))
                                    .cloned()
                                    .collect()
                            }
                        },
                        Err(e) => {
                            println!("Error parsing JSON: {}, falling back to all images", e);
                            files.iter()
                                .filter(|s| s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png"))
                                .cloned()
                                .collect()
                        }
                    }
                },
                Err(e) => {
                    println!("Error opening JSON file: {}, falling back to all images", e);
                    files.iter()
                        .filter(|s| s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png"))
                        .cloned()
                        .collect()
                }
            }
        },
        None => {
            println!("No JSON file found, using all images");
            files.iter()
                .filter(|s| s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png"))
                .cloned()
                .collect()
        }
    };
    
    // Make sure we have images
    if images.is_empty() {
      images = match files.iter().find(|s| s.to_lowercase().ends_with(".jpg") || s.to_lowercase().ends_with(".png")) {
          Some(filename) => {
              println!("Found image file: {}", filename);
              vec![filename.clone()]
          },
          None => {
              println!("No image file found");
              vec![]
          }
      };
    }
    if images.is_empty() {
      println!("No images found!");
      return;
    }
    
    println!("Found {} images", images.len());
    
    let events_loop = winit::event_loop::EventLoop::new();
    let monitor = events_loop.primary_monitor().unwrap();
    let monitor_size = monitor.size();
    
    let screen_width = monitor_size.width;
    let screen_height = monitor_size.height;
    
    let kauhanen_version = env!("CARGO_PKG_VERSION");
    
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new(format!("kauhanen {}", kauhanen_version), [screen_width, screen_height])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .fullscreen(true)
        .build()
        .unwrap();
    
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    
    // Load images and create textures
    let mut images_loaded: Vec<sprite::Sprite<_>> =
        images.iter().map(|s| {
            println!("Loading image: {}", s);
            std::rc::Rc::new(Texture::from_path(
                &mut texture_context,
                s,
                Flip::None,
                &TextureSettings::new()
            ).unwrap())
        })
        .map(|s|
            Sprite::from_texture(s.clone())
        )
        .collect();
    
    println!("Loaded {} images", images_loaded.len());
    
    // Center and scale sprites
    for s in &mut images_loaded {
        let (spr_x, spr_y) = s.get_texture().get_size();
        let scale_y = (screen_height as f64) / (spr_y as f64);
        let scale_x = (screen_width as f64) / (spr_x as f64);
        s.set_scale(scale_x, scale_y);
        s.set_position(screen_width as f64 / 2.0, screen_height as f64 / 2.0);
    }
    

    let file = File::open(music_file).expect("Failed to open music file");
    let source = Decoder::new(BufReader::new(file)).expect("Failed to decode audio");
    let music_length_ms = source.total_duration().unwrap().as_millis() as u32;

    let music_posses: Vec<u32> = (0..images.len())
        .map(|s| (s as u32 * music_length_ms / images.len() as u32))
        .collect();
    
    println!("Image transition points (ms): {:?}", music_posses);
    println!("Estimated music length: {} ms", music_length_ms);

    let mut last = 0;
    let mut current_sprite_index = 0;
    
    sink.append(source);
    sink.play();
    
    let start_time = Instant::now();
    
    while let Some(e) = window.next() {
        // Calculate current position in milliseconds
        let elapsed = start_time.elapsed();
        let pos = elapsed.as_millis() as u32;
        //println!("Current position: {} ms", pos);
        
        if last < music_posses.len() && pos >= music_posses[last] {
            current_sprite_index = last;
            last = last + 1;
            println!("Switching to image {} at {} ms", current_sprite_index, pos);
        }
        
        window.draw_2d(&e, |c, g, _| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            // Always draw a sprite if we have any
            if !images_loaded.is_empty() {
                // Make sure index is in bounds
                let index = current_sprite_index.min(images_loaded.len() - 1);
                images_loaded[index].draw(c.transform, g);
            }
        });
        
        if sink.empty() {
            println!("Music playback finished");
            break;
        }
    }
    
    // Stop playback
    sink.stop();
    println!("Exiting");
}

