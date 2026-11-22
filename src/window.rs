pub struct Window {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub normal_x: i32,
    pub normal_y: i32,
    pub normal_w: i32,
    pub normal_h: i32,
    pub title: &'static str,
    pub dragging: bool,
    pub resizing: bool,
    pub is_fullscreen: bool,
}

impl Window {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Window {
            x,
            y,
            w,
            h,
            normal_x: x,
            normal_y: y,
            normal_w: w,
            normal_h: h,
            title: "Plasma",
            dragging: false,
            resizing: false,
            is_fullscreen: false,
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        if self.is_fullscreen {
            self.x = self.normal_x;
            self.y = self.normal_y;
            self.w = self.normal_w;
            self.h = self.normal_h;
            self.is_fullscreen = false;
        } else {
            self.normal_x = self.x;
            self.normal_y = self.y;
            self.normal_w = self.w;
            self.normal_h = self.h;
            self.x = 0;
            self.y = 0;
            self.w = 320;
            self.h = 180;
            self.is_fullscreen = true;
        }
    }
}
