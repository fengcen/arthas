
pub struct Meta {
    pub offset: usize,
    pub real: usize,
    pub size: usize,
}

impl Meta {
    pub fn new(bytes: &[u8]) -> Meta {
        let size = bytes.len();
        Meta {
            offset: 0,
            real: size,
            size: size,
        }
    }

    pub fn to_simple(&self) -> SimpleMeta {
        SimpleMeta(self.offset, self.real, self.size)
    }
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMeta(usize, usize, usize);

impl SimpleMeta {
    pub fn set_offset(&mut self, v: usize) {
        self.0 = v;
    }

    pub fn set_real(&mut self, v: usize) {
        self.1 = v;
    }

    pub fn offset(&self) -> usize {
        self.0
    }

    pub fn real(&self) -> usize {
        self.1
    }

    pub fn size(&self) -> usize {
        self.2
    }
}
