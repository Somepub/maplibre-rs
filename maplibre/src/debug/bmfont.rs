use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BMChar {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub xoffset: f32,
    pub yoffset: f32,
    pub xadvance: f32,
}

#[derive(Debug, Clone)]
pub struct BMFont {
    pub line_height: f32,
    pub base: f32,
    pub scale_w: f32,
    pub scale_h: f32,
    pub chars: HashMap<u32, BMChar>,
}

impl BMFont {
    pub fn from_fnt(text: &str) -> Self {
        let mut line_height = 0.0;
        let mut base = 0.0;
        let mut scale_w = 0.0;
        let mut scale_h = 0.0;

        let mut chars = HashMap::<u32, BMChar>::new();

        for line in text.lines() {
            let line = line.trim();

            if line.starts_with("common ") {
                for part in line.split_whitespace() {
                    if let Some((k, v)) = part.split_once('=') {
                        match k {
                            "lineHeight" => line_height = v.parse::<f32>().unwrap(),
                            "base" => base = v.parse::<f32>().unwrap(),
                            "scaleW" => scale_w = v.parse::<f32>().unwrap(),
                            "scaleH" => scale_h = v.parse::<f32>().unwrap(),
                            _ => {}
                        }
                    }
                }
            }

            if line.starts_with("char ") {
                let mut id = 0;
                let mut x = 0.0;
                let mut y = 0.0;
                let mut w = 0.0;
                let mut h = 0.0;
                let mut xoffset = 0.0;
                let mut yoffset = 0.0;
                let mut xadvance = 0.0;

                for part in line.split_whitespace() {
                    if let Some((k, v)) = part.split_once('=') {
                        match k {
                            "id" => id = v.parse::<u32>().unwrap(),
                            "x" => x = v.parse::<f32>().unwrap(),
                            "y" => y = v.parse::<f32>().unwrap(),
                            "width" => w = v.parse::<f32>().unwrap(),
                            "height" => h = v.parse::<f32>().unwrap(),
                            "xoffset" => xoffset = v.parse::<f32>().unwrap(),
                            "yoffset" => yoffset = v.parse::<f32>().unwrap(),
                            "xadvance" => xadvance = v.parse::<f32>().unwrap(),
                            _ => {}
                        }
                    }
                }

                chars.insert(
                    id,
                    BMChar {
                        id,
                        x,
                        y,
                        w,
                        h,
                        xoffset,
                        yoffset,
                        xadvance,
                    },
                );
            }
        }

        BMFont {
            line_height,
            base,
            scale_w,
            scale_h,
            chars,
        }
    }
}
