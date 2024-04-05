#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageMapTable(u64);

impl PageMapTable {
    pub fn execute_disable(&self) -> bool {
        self.0 & (1 << 63) == 1 << 63
    }

    pub fn protection_key(&self) -> u8 {
        (self.0 >> 59) as u8
    }

    pub fn global(&self) -> bool {
        self.0 & (1 << 8) == 1 << 8
    }

    pub fn address(&self, max: u8) -> u64 {
        self.0 << max >> 12 + max
    }

    pub fn page_size(&self) -> bool {
        self.0 & (1 << 7) == (1 << 7)
    }

    pub fn written(&self) -> bool {
        self.0 & (1 << 6) == (1 << 6)
    }

    pub fn accessed(&self) -> bool {
        self.0 & (1 << 5) == (1 << 5)
    }

    pub fn cache_disable(&self) -> bool {
        self.0 & (1 << 4) == (1 << 4)
    }

    pub fn write_through(&self) -> bool {
        self.0 & (1 << 3) == (1 << 3)
    }

    pub fn user_allowed(&self) -> bool {
        self.0 & (1 << 2) == (1 << 2)
    }

    pub fn write_allowed(&self) -> bool {
        self.0 & (1 << 1) == (1 << 1)
    }

    pub fn present(&self) -> bool {
        self.0 & 1 == 1
    }
}

impl From<u64> for PageMapTable {
    fn from(value: u64) -> Self {
        PageMapTable(value)
    }
}

impl From<PageMapTableBuilder> for PageMapTable {
    fn from(value: PageMapTableBuilder) -> Self {
        PageMapTable::from(value.0)
    }
}

pub struct PageMapTableBuilder(u64);

impl PageMapTableBuilder {
    pub fn execute_disable(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1 << 63;
        } else {
            self.0 &= !(1 << 63);
        }

        self
    }

    pub fn protection_key(mut self, key: u64) -> PageMapTableBuilder {
        self.0 |= key << 59;

        self
    }

    pub fn address(mut self, address: u64) -> PageMapTableBuilder {
        self.0 |= address << 12;

        self
    }

    pub fn cache_disable(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1 << 4;
        } else {
            self.0 &= !(1 << 4);
        }

        self
    }

    pub fn write_through(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1 << 3;
        } else {
            self.0 &= !(1 << 3);
        }

        self
    }

    pub fn user_allowed(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1 << 2;
        } else {
            self.0 &= !(1 << 2);
        }

        self
    }

    pub fn write_allowed(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1 << 1;
        } else {
            self.0 &= !(1 << 1);
        }

        self
    }

    pub fn present(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1;
        } else {
            self.0 &= !1;
        }

        self
    }

    pub fn page_size(mut self, set: bool) -> PageMapTableBuilder {
        if set {
            self.0 |= 1 << 7;
        } else {
            self.0 &= !(1 << 7);
        }

        self
    }
}

impl From<PageMapTable> for PageMapTableBuilder {
    fn from(value: PageMapTable) -> Self {
        PageMapTableBuilder(value.0)
    }
}

impl From<u64> for PageMapTableBuilder {
    fn from(value: u64) -> Self {
        PageMapTableBuilder(value)
    }
}

impl From<PageMapTableBuilder> for u64 {
    fn from(value: PageMapTableBuilder) -> Self {
        value.0
    }
}
