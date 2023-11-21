#![deny(clippy::all)]
use napi_derive::napi;
use utiles_cli::cli_main_sync;
use utiles;



#[macro_use]
extern crate napi_derive;

#[napi(js_name = "Tile")]
pub struct JsTile {
  xyz: utiles::Tile,
}

#[napi]
impl JsTile {
    #[napi(constructor)]
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        JsTile {
            xyz: utiles::Tile::new(x, y, z),
        }
    }

    #[napi(getter)]
    pub fn x(&self) -> u32 {
        self.xyz.x
    }

    #[napi(getter)]
    pub fn y(&self) -> u32 {
        self.xyz.y
    }

    #[napi(getter)]
    pub fn z(&self) -> u8 {
        self.xyz.z
    }
}
#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn ut_cli(
  args: Vec<String>
) -> u8 {
  cli_main_sync(
     Some(args),
     None
  )
}
