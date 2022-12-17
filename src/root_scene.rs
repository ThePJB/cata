use glutin::event::VirtualKeyCode;
use crate::kapp::*;
use crate::kmath::*;

use crate::game::*;

pub struct RootScene {
    curr_scene: Option<Box<dyn Demo>>,
    show_menu: bool,
}

impl Default for RootScene {
    fn default() -> Self {
        RootScene {
            curr_scene: Some(init_demo::<Game>()),
            show_menu: false,
        }
    }
}

impl Demo for RootScene {
    fn frame(&mut self, inputs: &FrameInputs, outputs: &mut FrameOutputs) {
        if let Some(curr) = self.curr_scene.as_mut() {
            curr.frame(inputs, outputs);
        } else {
            self.show_menu = true;
        }

        // if inputs.key_rising(VirtualKeyCode::Escape) {
        //     self.show_menu = !self.show_menu;
        // }
        // if self.show_menu {
        //     let ca = 12.0 / 14.0;
        //     let ch = 0.02;
        //     let cw = ch * ca;
        //     let w = cw * 30.0;

        //     let c = Vec4::grey(0.3);
        //     let wcom = 1.0 - w;
        //     outputs.canvas.put_rect(inputs.screen_rect.child(wcom/2.0, 0.0, w, 1.0), 4.0, c);

            // outputs.glyphs.push_center_str(self.demo_table[i].0, x, y_initial + i as f32 * ch, cw, ch * 0.8, 5.5, colour);
        // }
    }
}