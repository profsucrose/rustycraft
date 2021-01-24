pub struct FaceUVs {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32
}

impl FaceUVs {
    pub fn new(left: f32, bottom: f32, right: f32, top: f32, width: f32, height: f32) -> FaceUVs {
        FaceUVs {
            left: left / width,
            bottom: bottom / height,
            right: right / width,
            top: top / width
        }
    }
}