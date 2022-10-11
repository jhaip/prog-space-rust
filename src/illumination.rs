use mlua::{MetaMethod, Table, UserData, UserDataFields};
use serde_json::{json, Result, Value};

pub struct Illumination {
    // TODO: graphics: JSON_type
    pub graphics: Vec<serde_json::Value>,
}
impl Illumination {
    fn get_int_from_lua_table(opts: &Table, key: &str, fallback: i32) -> i32 {
        match opts.get(key) {
            Ok(v) => v,
            Err(_) => fallback,
        }
    }
    fn get_float_from_lua_table(opts: &Table, key: &str, fallback: f64) -> f64 {
        match opts.get(key) {
            Ok(v) => v,
            Err(_) => fallback,
        }
    }
    fn get_color_from_lua_table(opts: &Table, key: &str, fallback: Vec<i32>) -> Vec<i32> {
        match opts.get::<&str, Table>(key) {
            Ok(t) => {
                let mut color = vec![0, 0, 0, 255];
                if let Ok(v) = t.get(1) {
                    color[0] = v;
                }
                if let Ok(v) = t.get(2) {
                    color[1] = v;
                }
                if let Ok(v) = t.get(3) {
                    color[2] = v;
                }
                if let Ok(v) = t.get(4) {
                    color[3] = v;
                }
                color
            }
            Err(_) => fallback,
        }
    }
}
impl UserData for Illumination {
    // fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
    //     fields.add_field_method_get("val", |_, this| Ok(this.0));
    // }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("rectangle", |_, this, opts: Table| {
            this.graphics.push(json!({
                "type": "rectangle",
                "options": {
                    "x": Illumination::get_int_from_lua_table(&opts, "x", 0),
                    "y": Illumination::get_int_from_lua_table(&opts, "y", 0),
                    "w": Illumination::get_int_from_lua_table(&opts, "w", 10),
                    "h": Illumination::get_int_from_lua_table(&opts, "h", 10),
                    "fill": Illumination::get_color_from_lua_table(&opts, "fill", vec![255, 255, 255, 255]),
                    "stroke": Illumination::get_color_from_lua_table(&opts, "stroke", vec![255, 255, 255, 255]),
                    "stroke_width": Illumination::get_int_from_lua_table(&opts, "stroke_width", 1)
                }
            }));
            Ok(())
        });
        methods.add_method_mut("ellipse", |_, this, opts: Table| {
            this.graphics.push(json!({
                "type": "ellipse",
                "options": {
                    "x": Illumination::get_int_from_lua_table(&opts, "x", 0),
                    "y": Illumination::get_int_from_lua_table(&opts, "y", 0),
                    "w": Illumination::get_int_from_lua_table(&opts, "w", 10),
                    "h": Illumination::get_int_from_lua_table(&opts, "h", 10),
                    "fill": Illumination::get_color_from_lua_table(&opts, "fill", vec![255, 255, 255, 255]),
                    "stroke": Illumination::get_color_from_lua_table(&opts, "stroke", vec![255, 255, 255, 255]),
                    "stroke_width": Illumination::get_int_from_lua_table(&opts, "stroke_width", 1)
                }
            }));
            Ok(())
        });
        methods.add_method_mut("line", |_, this, opts: Table| {
            this.graphics.push(json!({
                "type": "line",
                "options": {
                    "x1": Illumination::get_int_from_lua_table(&opts, "x1", 0),
                    "y1": Illumination::get_int_from_lua_table(&opts, "y1", 0),
                    "x2": Illumination::get_int_from_lua_table(&opts, "x2", 0),
                    "y2": Illumination::get_int_from_lua_table(&opts, "y2", 0),
                    "color": Illumination::get_color_from_lua_table(&opts, "color", vec![255, 255, 255, 255]),
                    "thickness": Illumination::get_int_from_lua_table(&opts, "thickness", 1)
                }
            }));
            Ok(())
        });
        methods.add_method_mut("text", |_, this, opts: Table| {
            this.graphics.push(json!({
                "type": "text",
                "options": {
                    "x": Illumination::get_int_from_lua_table(&opts, "x", 0),
                    "y": Illumination::get_int_from_lua_table(&opts, "y", 0),
                    "text": match opts.get("text") {
                        Ok(t) => t,
                        _ => "".to_string()
                    },
                    "color": Illumination::get_color_from_lua_table(&opts, "color", vec![255, 255, 255, 255]),
                    "size": Illumination::get_int_from_lua_table(&opts, "size", 12)
                }
            }));
            Ok(())
        });
        methods.add_method_mut("frame", |_, this, opts: Table| {
            this.graphics.push(json!({
                "type": "frame",
                "options": {
                    "x": Illumination::get_int_from_lua_table(&opts, "x1", 0),
                    "y": Illumination::get_int_from_lua_table(&opts, "y1", 0),
                    "scale": Illumination::get_float_from_lua_table(&opts, "scale", 1.0),
                    "clip_x": Illumination::get_int_from_lua_table(&opts, "clip_x", 0),
                    "clip_y": Illumination::get_int_from_lua_table(&opts, "clip_y", 0),
                    "clip_w": Illumination::get_int_from_lua_table(&opts, "clip_w", -1),
                    "clip_h": Illumination::get_int_from_lua_table(&opts, "clip_h", -1),
                }
            }));
            Ok(())
        });
        methods.add_method_mut("image", |_, this, opts: Table| {
            this.graphics.push(json!({
                "type": "image",
                "options": {
                    "x": Illumination::get_int_from_lua_table(&opts, "x", 0),
                    "y": Illumination::get_int_from_lua_table(&opts, "y", 0),
                    "scale": Illumination::get_float_from_lua_table(&opts, "scale", 1.0),
                    "filepath": match opts.get("filepath") {
                        Ok(t) => t,
                        _ => "".to_string()
                    }
                }
            }));
            Ok(())
        });

        methods.add_function("new", |_, ()| Ok(Illumination { graphics: vec![] }));

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            let s = this
                .graphics
                .iter()
                .map(|v| serde_json::to_string(v).unwrap())
                .collect::<Vec<String>>()
                .join(",");
            Ok(format!("[{}]", s).to_string())
        });

        // Constructor
        methods.add_meta_function(MetaMethod::Call, |_, ()| {
            Ok(Illumination { graphics: vec![] })
        });
    }
}
