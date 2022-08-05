use {
    crate::{Bytes, BytesRev, Chars, CharsRev, Chunks, ChunksRev, Cursor, Info, Rope},
    std::ops::RangeBounds,
};

#[derive(Clone, Copy, Debug)]
pub struct Slice<'a> {
    rope: &'a Rope,
    start_info: Info,
    end_info: Info,
}

impl<'a> Slice<'a> {
    pub fn is_empty(self) -> bool {
        self.byte_len() == 0
    }

    pub fn byte_len(self) -> usize {
        self.end_info.byte_count - self.start_info.byte_count
    }

    pub fn char_len(self) -> usize {
        self.end_info.char_count - self.start_info.char_count
    }

    pub fn line_len(self) -> usize {
        self.end_info.line_break_count - self.start_info.line_break_count + 1
    }

    pub fn byte_to_char(self, byte_index: usize) -> usize {
        self.info_at(byte_index).char_count
    }

    pub fn byte_to_line(self, byte_index: usize) -> usize {
        self.info_at(byte_index).line_break_count + 1
    }

    pub fn char_to_byte(self, char_index: usize) -> usize {
        if char_index == 0 {
            return 0;
        }
        if char_index == self.char_len() {
            return self.byte_len();
        }
        self.rope
            .char_to_byte(self.start_info.char_count + char_index)
            - self.start_info.byte_count
    }

    pub fn line_to_byte(self, line_index: usize) -> usize {
        if line_index == 0 {
            return 0;
        }
        self.rope
            .line_to_byte(self.start_info.line_break_count + line_index)
            - self.start_info.byte_count
    }

    pub fn slice<R: RangeBounds<usize>>(&self, byte_range: R) -> Slice<'_> {
        let byte_range = crate::range_bounds_to_range(byte_range, self.byte_len());
        Slice::new(
            &self.rope,
            self.start_info.byte_count + byte_range.start,
            self.start_info.byte_count + byte_range.end,
        )
    }

    pub fn cursor_front(self) -> Cursor<'a> {
        Cursor::front(
            self.rope.root(),
            self.start_info.byte_count,
            self.end_info.byte_count,
        )
    }

    pub fn cursor_back(self) -> Cursor<'a> {
        Cursor::back(
            self.rope.root(),
            self.start_info.byte_count,
            self.end_info.byte_count,
        )
    }

    pub fn cursor_at(self, byte_index: usize) -> Cursor<'a> {
        Cursor::at(
            self.rope.root(),
            self.start_info.byte_count,
            self.end_info.byte_count,
            byte_index,
        )
    }

    pub fn chunks(self) -> Chunks<'a> {
        Chunks::new(self)
    }

    pub fn chunks_rev(self) -> ChunksRev<'a> {
        ChunksRev::new(self)
    }

    pub fn bytes(self) -> Bytes<'a> {
        Bytes::new(self)
    }

    pub fn bytes_rev(self) -> BytesRev<'a> {
        BytesRev::new(self)
    }

    pub fn chars(self) -> Chars<'a> {
        Chars::new(self)
    }

    pub fn chars_rev(self) -> CharsRev<'a> {
        CharsRev::new(self)
    }

    pub(crate) fn new(rope: &'a Rope, byte_start: usize, byte_end: usize) -> Self {
        use crate::StrUtils;

        let (chunk, mut start_info) = rope.root().chunk_at_byte(byte_start);
        start_info += Info::from(&chunk[..byte_start - start_info.byte_count]);
        if chunk[..byte_start].last_is_cr() && chunk[byte_start..].first_is_lf() {
            start_info.line_break_count -= 1;
        }
        Self {
            rope,
            start_info,
            end_info: rope.info_at(byte_end),
        }
    }

    pub(crate) fn info_at(&self, byte_index: usize) -> Info {
        if byte_index == 0 {
            return Info::new();
        }
        if byte_index == self.byte_len() {
            return self.end_info - self.start_info;
        }
        self.rope.info_at(self.start_info.byte_count + byte_index) - self.start_info
    }
}