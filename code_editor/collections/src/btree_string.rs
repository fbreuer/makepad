use {
    crate::{btree, BTree},
    std::{cmp::Ordering, ops::{Add, AddAssign, Range, RangeBounds, Sub, SubAssign}, str},
};

#[derive(Clone)]
pub struct BTreeString {
    btree: BTree<String, Info>,
}

impl BTreeString {
    pub fn new() -> Self {
        Self {
            btree: BTree::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.btree.is_empty()
    }

    pub fn len(&self) -> usize {
        self.btree.len()
    }

    pub fn char_len(&self) -> usize {
        self.btree.info().char_count
    }

    pub fn line_len(&self) -> usize {
        self.btree.info().line_break_count + 1
    }

    pub fn is_char_boundary(&self, index: usize) -> bool {
        if index > self.len() {
            return false;
        }
        self.cursor_at(index).is_at_char_boundary()
    }

    pub fn index_to_char_index(&self, index: usize) -> usize {
        self.btree.index_to_info(index).char_count
    }

    pub fn index_to_line_index(&self, index: usize) -> usize {
        self.btree.index_to_info(index).line_break_count
    }

    pub fn char_index_to_index(&self, char_index: usize) -> usize {
        if char_index == 0 {
            return 0;
        }
        match self
            .btree
            .search_by(|_, total_info| char_index < total_info.char_count)
        {
            Some((chunk, total_len, total_info)) => {
                total_len + chunk.char_index_to_index(char_index - total_info.char_count)
            }
            None => self.len(),
        }
    }

    pub fn line_index_to_index(&self, line_index: usize) -> usize {
        if line_index == 0 {
            return 0;
        }
        match self
            .btree
            .search_by(|_, total_info| line_index <= total_info.line_break_count)
        {
            Some((chunk, total_len, total_info)) => {
                total_len + chunk.line_index_to_index(line_index - total_info.line_break_count)
            }
            None => panic!(),
        }
    }

    pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> Slice<'_> {
        Slice {
            slice: self.btree.slice(range),
        }
    }

    pub fn cursor_front(&self) -> Cursor<'_> {
        self.slice(..).cursor_front()
    }

    pub fn cursor_back(&self) -> Cursor<'_> {
        self.slice(..).cursor_back()
    }

    pub fn cursor_at(&self, position: usize) -> Cursor<'_> {
        self.slice(..).cursor_at(position)
    }

    pub fn chunks(&self) -> Chunks<'_> {
        self.slice(..).chunks()
    }

    pub fn chunks_rev(&self) -> ChunksRev<'_> {
        self.slice(..).chunks_rev()
    }

    pub fn bytes(&self) -> Bytes<'_> {
        self.slice(..).bytes()
    }

    pub fn bytes_rev(&self) -> BytesRev<'_> {
        self.slice(..).bytes_rev()
    }

    pub fn chars(&self) -> Chars<'_> {
        self.slice(..).chars()
    }

    pub fn chars_rev(&self) -> CharsRev<'_> {
        self.slice(..).chars_rev()
    }

    pub fn replace_range<R: RangeBounds<usize>>(&mut self, range: R, replace_with: Self) {
        let range = btree::range(range, self.len());
        if range.is_empty() {
            let other = self.split_off(range.start);
            self.append(replace_with);
            self.append(other);
        } else {
            let mut other = self.clone();
            self.truncate_back(range.start);
            other.truncate_front(range.end);
            self.append(replace_with);
            self.append(other);
        }
    }

    pub fn append(&mut self, mut other: Self) {
        let chunk_0 = self.cursor_back().current_chunk();
        let chunk_1 = other.cursor_front().current_chunk();
        match (chunk_0.as_bytes().last(), chunk_1.as_bytes().first()) {
            (Some(0x0D), Some(0x0A)) => {
                self.btree.truncate_back(self.len() - 1);
                other.btree.truncate_front(1);
                self.btree.append(BTree::from(String::from("\r\n")));
                self.btree.append(other.btree);
            }
            _ => self.btree.append(other.btree),
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            btree: self.btree.split_off(at),
        }
    }

    pub fn truncate_front(&mut self, start: usize) {
        self.btree.truncate_front(start)
    }

    pub fn truncate_back(&mut self, end: usize) {
        self.btree.truncate_back(end)
    }
}

impl Eq for BTreeString {}

impl Ord for BTreeString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.slice(..).cmp(&other.slice(..))
    }
}

impl PartialEq for BTreeString {
    fn eq(&self, other: &Self) -> bool {
        self.slice(..).eq(&other.slice(..))
    }
}

impl<'a> PartialEq<Slice<'a>> for BTreeString {
    fn eq(&self, other: &Slice<'a>) -> bool {
        self.slice(..).eq(other)
    }
}

impl PartialOrd for BTreeString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.slice(..).partial_cmp(&other.slice(..))
    }
}

impl<'a> PartialOrd<Slice<'a>> for BTreeString {
    fn partial_cmp(&self, other: &Slice<'a>) -> Option<Ordering> {
        self.slice(..).partial_cmp(other)
    }
}

impl From<String> for BTreeString {
    fn from(string: String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&String> for BTreeString {
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&str> for BTreeString {
    fn from(string: &str) -> Self {
        let mut builder = Builder::new();
        builder.push_chunk(string);
        builder.build()
    }
}

pub struct Builder {
    builder: btree::Builder<String, Info>,
    chunk: String,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            builder: btree::Builder::new(),
            chunk: String::new(),
        }
    }

    pub fn push_chunk(&mut self, mut chunk: &str) {
        while !chunk.is_empty() {
            if chunk.len() <= <String as btree::Chunk>::MAX_LEN - self.chunk.len() {
                self.chunk.push_str(chunk);
                break;
            }
            let mut index = <String as btree::Chunk>::MAX_LEN - self.chunk.len();
            while !chunk.is_char_boundary(index) {
                index -= 1;
            }
            let (left_chunk, right_chunk) = chunk.split_at(index);
            self.chunk.push_str(left_chunk);
            chunk = right_chunk;
            self.builder.push_chunk(self.chunk.split_off(0));
        }
    }

    pub fn build(mut self) -> BTreeString {
        self.builder.push_chunk(self.chunk);
        BTreeString {
            btree: self.builder.build(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Slice<'a> {
    slice: btree::Slice<'a, String, Info>,
}

impl<'a> Slice<'a> {
    pub fn to_btree_string(self) -> BTreeString {
        BTreeString {
            btree: self.slice.to_btree(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.slice.is_empty()
    }

    pub fn len(self) -> usize {
        self.slice.len()
    }

    pub fn char_len(self) -> usize {
        self.slice.info().char_count
    }

    pub fn line_len(self) -> usize {
        self.slice.info().line_break_count + 1
    }

    pub fn is_char_boundary(self, index: usize) -> bool {
        if index > self.len() {
            return false;
        }
        self.cursor_at(index).is_at_char_boundary()
    }

    pub fn index_to_char_index(self, index: usize) -> usize {
        self.slice.index_to_info(index).char_count
    }

    pub fn index_to_line_index(self, index: usize) -> usize {
        self.slice.index_to_info(index).line_break_count
    }

    pub fn char_index_to_index(self, char_index: usize) -> usize {
        if char_index == 0 {
            return 0;
        }
        match self
            .slice
            .search_by(|_, total_info| char_index < total_info.char_count)
        {
            Some((chunk, range, total_len, total_info)) => {
                let chunk = &chunk[range];
                total_len + chunk.char_index_to_index(char_index - total_info.char_count)
            }
            None => self.len(),
        }
    }

    pub fn line_index_to_index(self, line_index: usize) -> usize {
        if line_index == 0 {
            return 0;
        }
        match self
            .slice
            .search_by(|_, total_info| line_index <= total_info.line_break_count)
        {
            Some((chunk, range, total_len, total_info)) => {
                let chunk = &chunk[range];
                total_len + chunk.line_index_to_index(line_index - total_info.line_break_count)
            }
            None => panic!(),
        }
    }

    pub fn cursor_front(self) -> Cursor<'a> {
        let cursor = self.slice.cursor_front();
        let (current, range) = cursor.current();
        Cursor {
            cursor,
            current: &current[range],
            index: 0,
        }
    }

    pub fn cursor_back(self) -> Cursor<'a> {
        let cursor = self.slice.cursor_back();
        let (current, range) = cursor.current();
        let current = &current[range];
        let index = current.len();
        Cursor {
            cursor,
            current,
            index,
        }
    }

    pub fn cursor_at(self, position: usize) -> Cursor<'a> {
        let cursor = self.slice.cursor_at(position);
        let (current, range) = cursor.current();
        let current = &current[range];
        let index = position - cursor.position();
        Cursor {
            cursor,
            current,
            index,
        }
    }

    pub fn chunks(self) -> Chunks<'a> {
        Chunks {
            cursor: self.cursor_front(),
        }
    }

    pub fn chunks_rev(&self) -> ChunksRev<'a> {
        ChunksRev {
            cursor: self.cursor_back(),
        }
    }

    pub fn bytes(self) -> Bytes<'a> {
        Bytes {
            bytes: None,
            chunks: self.chunks(),
        }
    }

    pub fn bytes_rev(self) -> BytesRev<'a> {
        BytesRev {
            bytes: None,
            chunks_rev: self.chunks_rev(),
        }
    }

    pub fn chars(self) -> Chars<'a> {
        Chars {
            chars: None,
            chunks: self.chunks(),
        }
    }
    
    pub fn chars_rev(&self) -> CharsRev<'a> {
        CharsRev {
            chars: None,
            chunks_rev: self.chunks_rev(),
        }
    }
}

impl<'a> Eq for Slice<'a> {}

impl<'a> Ord for Slice<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut chunks_0 = self.chunks();
        let mut chunks_1 = other.chunks();
        let mut chunk_0 = chunks_0.next().unwrap_or("").as_bytes();
        let mut chunk_1 = chunks_1.next().unwrap_or("").as_bytes();
        loop {
            match chunk_0.len().cmp(&chunk_1.len()) {
                Ordering::Less => {
                    let len = chunk_0.len();
                    if len == 0 {
                        break Ordering::Less;
                    }
                    let cmp = chunk_0.cmp(&chunk_1[..len]);
                    if cmp != Ordering::Equal {
                        break cmp;
                    }
                    chunk_0 = chunks_0.next().unwrap_or("").as_bytes();
                    chunk_1 = &chunk_1[len..];
                }
                Ordering::Equal => {
                    if chunk_0.len() == 0 {
                        break Ordering::Equal;
                    }
                    let cmp = chunk_0.cmp(&chunk_1);
                    if cmp != Ordering::Equal {
                        break cmp;
                    }
                    chunk_0 = chunks_0.next().unwrap_or("").as_bytes();
                    chunk_1 = chunks_1.next().unwrap_or("").as_bytes();
                }
                Ordering::Greater => {
                    let len = chunk_1.len();
                    if len == 0 {
                        break Ordering::Greater;
                    }
                    let cmp = chunk_0[..len].cmp(&chunk_1);
                    if cmp != Ordering::Equal {
                        break cmp;
                    }
                    chunk_0 = &chunk_0[len..];
                    chunk_1 = chunks_1.next().unwrap_or("").as_bytes();
                }
            }
        }
    }
}

impl<'a> PartialEq for Slice<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> PartialEq<BTreeString> for Slice<'a> {
    fn eq(&self, other: &BTreeString) -> bool {
        self.eq(&other.slice(..))
    }
}

impl<'a> PartialOrd for Slice<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialOrd<BTreeString> for Slice<'a> {
    fn partial_cmp(&self, other: &BTreeString) -> Option<Ordering> {
        self.partial_cmp(&other.slice(..))
    }
}

#[derive(Clone)]
pub struct Cursor<'a> {
    cursor: btree::Cursor<'a, String, Info>,
    current: &'a str,
    index: usize,
}

impl<'a> Cursor<'a> {
    pub fn is_at_front(&self) -> bool {
        self.index == 0 && self.cursor.is_at_front()
    }

    pub fn is_at_back(&self) -> bool {
        self.index == self.current.len()
    }

    pub fn is_at_char_boundary(&self) -> bool {
        self.current.is_char_boundary(self.index)
    }

    pub fn position(&self) -> usize {
        self.cursor.position() + self.index
    }

    pub fn current_chunk(&self) -> &'a str {
        self.current
    }

    pub fn current_byte(&self) -> u8 {
        self.current.as_bytes()[self.index]
    }

    pub fn current_char(&self) -> char {
        self.current[self.index..].chars().next().unwrap()
    }

    pub fn move_next_chunk(&mut self) {
        if self.cursor.is_at_back() {
            self.index = self.current.len();
            return;
        }
        self.move_next();
        self.index = 0;
    }

    pub fn move_prev_chunk(&mut self) {
        if self.index == self.current.len() {
            self.index = 0;
            return;
        }
        self.move_prev();
        self.index = 0;
    }

    pub fn move_next_byte(&mut self) {
        self.index += 1;
        if self.index == self.current.len() && !self.cursor.is_at_back() {
            self.move_next();
            self.index = 0;
        }
    }

    pub fn move_prev_byte(&mut self) {
        if self.index == 0 {
            self.move_prev();
            self.index = self.current.len();
        }
        self.index -= 1;
    }

    pub fn move_next_char(&mut self) {
        self.index += self.current_byte().utf8_char_len();
        if self.index == self.current.len() && !self.cursor.is_at_back() {
            self.move_next();
            self.index = 0;
        }
    }

    pub fn move_prev_char(&mut self) {
        if self.index == 0 {
            self.move_prev();
            self.index = self.current.len();
        }
        self.index -= 1;
        while !self.is_at_char_boundary() {
            self.index -= 1;
        }
    }

    fn move_next(&mut self) {
        self.cursor.move_next();
        let (current, range) = self.cursor.current();
        self.current = &current[range];
    }

    fn move_prev(&mut self) {
        self.cursor.move_prev();
        let (current, range) = self.cursor.current();
        self.current = &current[range];
    }
}

#[derive(Clone)]
pub struct Chunks<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Iterator for Chunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_at_back() {
            return None;
        }
        let chunk = self.cursor.current_chunk();
        self.cursor.move_next_chunk();
        Some(chunk)
    }
}

#[derive(Clone)]
pub struct ChunksRev<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Iterator for ChunksRev<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_at_front() {
            return None;
        }
        self.cursor.move_prev_chunk();
        Some(self.cursor.current_chunk())
    }
}

#[derive(Clone)]
pub struct Bytes<'a> {
    bytes: Option<str::Bytes<'a>>,
    chunks: Chunks<'a>,
}

impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.bytes {
                Some(bytes) => match bytes.next() {
                    Some(byte) => break Some(byte),
                    None => {
                        self.bytes = None;
                        continue;
                    }
                }
                None => {
                    match self.chunks.next() {
                        Some(chunk) => {
                            self.bytes = Some(chunk.bytes());
                            continue;
                        },
                        None => break None,
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct BytesRev<'a> {
    bytes: Option<str::Bytes<'a>>,
    chunks_rev: ChunksRev<'a>,
}

impl<'a> Iterator for BytesRev<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.bytes {
                Some(bytes) => match bytes.next_back() {
                    Some(byte) => break Some(byte),
                    None => {
                        self.bytes = None;
                        continue;
                    }
                }
                None => {
                    match self.chunks_rev.next() {
                        Some(chunk) => {
                            self.bytes = Some(chunk.bytes());
                            continue;
                        },
                        None => break None,
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Chars<'a> {
    chars: Option<str::Chars<'a>>,
    chunks: Chunks<'a>,
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.chars {
                Some(chars) => match chars.next() {
                    Some(ch) => break Some(ch),
                    None => {
                        self.chars = None;
                        continue;
                    }
                }
                None => {
                    match self.chunks.next() {
                        Some(chunk) => {
                            self.chars = Some(chunk.chars());
                            continue;
                        },
                        None => break None,
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct CharsRev<'a> {
    chars: Option<str::Chars<'a>>,
    chunks_rev: ChunksRev<'a>,
}

impl<'a> Iterator for CharsRev<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.chars {
                Some(chars) => match chars.next_back() {
                    Some(ch) => break Some(ch),
                    None => {
                        self.chars = None;
                        continue;
                    }
                }
                None => {
                    match self.chunks_rev.next() {
                        Some(chunk) => {
                            self.chars = Some(chunk.chars());
                            continue;
                        },
                        None => break None,
                    }
                }
            }
        }
    }
}

impl btree::Chunk for String {
    #[cfg(not(test))]
    const MAX_LEN: usize = 1024;
    #[cfg(test)]
    const MAX_LEN: usize = 8;

    fn len(&self) -> usize {
        self.len()
    }

    fn is_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.len() {
            return true;
        }
        let bytes = self.as_bytes();
        bytes[index].is_utf8_char_boundary() && bytes[index - 1] != 0x0D && bytes[index] != 0x0F
    }

    fn shift_left(&mut self, other: &mut Self, end: usize) {
        self.push_str(&other[..end]);
        other.replace_range(..end, "");
    }

    fn shift_right(&mut self, other: &mut Self, start: usize) {
        other.replace_range(..0, &self[start..]);
        self.truncate(start);
    }

    fn truncate_front(&mut self, start: usize) {
        self.replace_range(..start, "");
    }

    fn truncate_back(&mut self, end: usize) {
        self.truncate(end)
    }
}

#[derive(Clone, Copy)]
pub struct Info {
    char_count: usize,
    line_break_count: usize,
}

impl btree::Info<String> for Info {
    fn from_chunk_and_range(string: &String, range: Range<usize>) -> Self {
        Self {
            char_count: string[range.clone()].count_chars(),
            line_break_count: string[range].count_line_breaks(),
        }
    }
}

impl Add for Info {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            char_count: self.char_count + other.char_count,
            line_break_count: self.line_break_count + other.line_break_count,
        }
    }
}

impl AddAssign for Info {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Default for Info {
    fn default() -> Self {
        Self {
            char_count: 0,
            line_break_count: 0,
        }
    }
}

impl Sub for Info {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            char_count: self.char_count - other.char_count,
            line_break_count: self.line_break_count - other.line_break_count,
        }
    }
}

impl SubAssign for Info {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

trait U8Ext {
    fn is_utf8_char_boundary(self) -> bool;

    fn utf8_char_len(self) -> usize;
}

impl U8Ext for u8 {
    fn is_utf8_char_boundary(self) -> bool {
        (self as i8) >= -0x40
    }

    fn utf8_char_len(self) -> usize {
        if self < 0x80 {
            1
        } else if self < 0xE0 {
            2
        } else if self < 0xF0 {
            3
        } else {
            4
        }
    }
}

trait StrExt {
    fn count_chars(&self) -> usize;
    fn count_line_breaks(&self) -> usize;
    fn char_index_to_index(&self, char_index: usize) -> usize;
    fn line_index_to_index(&self, line_index: usize) -> usize;
}

impl StrExt for str {
    fn count_chars(&self) -> usize {
        let mut count = 0;
        for byte in self.bytes() {
            if byte.is_utf8_char_boundary() {
                count += 1;
            }
        }
        count
    }

    fn count_line_breaks(&self) -> usize {
        let mut count = 0;
        for byte in self.bytes() {
            if byte == 0x0A {
                count += 1;
            }
        }
        count
    }

    fn char_index_to_index(&self, char_index: usize) -> usize {
        let mut char_count = 0;
        let bytes = self.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index].is_utf8_char_boundary() {
                char_count += 1;
            }
            if char_count > char_index {
                break;
            }
            index += 1;
        }
        index
    }

    fn line_index_to_index(&self, line_index: usize) -> usize {
        let mut line_break_count = 0;
        let bytes = self.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == 0x0A {
                line_break_count += 1;
            }
            if line_break_count >= line_index {
                break;
            }
            index += 1;
        }
        index
    }
}

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*, std::ops::Range};

    fn string() -> impl Strategy<Value = String> {
        "(.|[\\n])*"
    }

    fn string_and_unaligned_index() -> impl Strategy<Value = (String, usize)> {
        string().prop_flat_map(|string| {
            let string_len = string.len();
            (Just(string), 0..=string_len)
        })
    }

    fn string_and_index() -> impl Strategy<Value = (String, usize)> {
        string_and_unaligned_index().prop_map(|(string, mut index)| {
            while !string.is_char_boundary(index) {
                index -= 1;
            }
            (string, index)
        })
    }

    fn string_and_char_index() -> impl Strategy<Value = (String, usize)> {
        string().prop_flat_map(|string| {
            let char_count = string.count_chars();
            (Just(string), 0..=char_count)
        })
    }

    fn string_and_line_index() -> impl Strategy<Value = (String, usize)> {
        string().prop_flat_map(|string| {
            let line_break_count = string.count_line_breaks();
            (Just(string), 0..=line_break_count)
        })
    }

    fn string_and_range() -> impl Strategy<Value = (String, Range<usize>)> {
        string_and_index()
            .prop_flat_map(|(string, end)| (Just(string), 0..=end, Just(end)))
            .prop_map(|(string, mut start, end)| {
                while !string.is_char_boundary(start) {
                    start -= 1;
                }
                (string, start..end)
            })
    }

    fn string_and_range_and_unaligned_index() -> impl Strategy<Value = (String, Range<usize>, usize)>
    {
        string_and_range().prop_flat_map(|(string, range)| {
            let range_len = range.len();
            (Just(string), Just(range), 0..=range_len)
        })
    }

    fn string_and_range_and_index() -> impl Strategy<Value = (String, Range<usize>, usize)> {
        string_and_range_and_unaligned_index().prop_map(|(string, range, mut index)| {
            let slice = &string[range.clone()];
            while !slice.is_char_boundary(index) {
                index -= 1;
            }
            (string, range, index)
        })
    }

    fn string_and_range_and_char_index() -> impl Strategy<Value = (String, Range<usize>, usize)> {
        string_and_range().prop_flat_map(|(string, range)| {
            let char_count = string[range.clone()].count_chars();
            (Just(string), Just(range), 0..=char_count)
        })
    }

    fn string_and_range_and_line_index() -> impl Strategy<Value = (String, Range<usize>, usize)> {
        string_and_range().prop_flat_map(|(string, range)| {
            let line_break_count = string[range.clone()].count_line_breaks();
            (Just(string), Just(range), 0..=line_break_count)
        })
    }

    proptest! {
        #[test]
        fn is_empty(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.is_empty(), string.is_empty());
        }

        #[test]
        fn len(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.len(), string.len());
        }

        #[test]
        fn char_len(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.char_len(), string.count_chars());
        }

        #[test]
        fn line_len(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.line_len(), string.count_line_breaks() + 1);
        }

        #[test]
        fn is_char_boundary((string, index) in string_and_unaligned_index()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.is_char_boundary(index), string.is_char_boundary(index));
        }

        #[test]
        fn index_to_char_index((string, index) in string_and_index()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.index_to_char_index(index), string[..index].count_chars());
        }

        #[test]
        fn index_to_line_index((string, index) in string_and_index()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.index_to_line_index(index),
                string[..index].count_line_breaks(),
            );
        }

        #[test]
        fn char_index_to_index((string, char_index) in string_and_char_index()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.char_index_to_index(char_index),
                string.char_index_to_index(char_index),
            );
        }

        #[test]
        fn line_index_to_index((string, line_index) in string_and_line_index()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.line_index_to_index(line_index),
                string.line_index_to_index(line_index),
            );
        }

        #[test]
        fn chunks(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(btree_string.chunks().collect::<String>(), string);
        }

        #[test]
        fn chunks_rev(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string
                    .chunks_rev()
                    .map(|chunk| chunk.chars().rev().collect::<String>())
                    .collect::<String>(),
                string.chars().rev().collect::<String>(),
            );
        }

        #[test]
        fn bytes(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.bytes().collect::<Vec<_>>(),
                string.bytes().collect::<Vec<_>>()
            );
        }

        #[test]
        fn bytes_rev(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.bytes_rev().collect::<Vec<_>>(),
                string.bytes().rev().collect::<Vec<_>>()
            );
        }

        #[test]
        fn chars(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.chars().collect::<Vec<_>>(),
                string.chars().collect::<Vec<_>>()
            );
        }

        #[test]
        fn chars_rev(string in string()) {
            let btree_string = BTreeString::from(&string);
            assert_eq!(
                btree_string.chars_rev().collect::<Vec<_>>(),
                string.chars().rev().collect::<Vec<_>>()
            );
        }

        #[test]
        fn replace_range((mut string, range) in string_and_range(), replace_with in string()) {
            let mut btree_string = BTreeString::from(&string);
            let replace_with_btree = BTreeString::from(&replace_with);
            btree_string.replace_range(range.clone(), replace_with_btree);
            string.replace_range(range, &replace_with);
            assert_eq!(btree_string.chunks().collect::<String>(), string);
        }

        #[test]
        fn append(mut string_0 in string(), string_1 in string()) {
            let mut btree_string_0 = BTreeString::from(&string_0);
            let btree_string_1 = BTreeString::from(&string_1);
            btree_string_0.append(btree_string_1);
            string_0.push_str(&string_1);
            assert_eq!(btree_string_0.chunks().collect::<String>(), string_0);
        }

        #[test]
        fn split_off((mut string, at) in string_and_index()) {
            let mut btree_string = BTreeString::from(&string);
            let other_string = string.split_off(at);
            let other_btree_string = btree_string.split_off(at);
            assert_eq!(btree_string.chunks().collect::<String>(), string);
            assert_eq!(other_btree_string.chunks().collect::<String>(), other_string);
        }

        #[test]
        fn truncate_front((mut string, start) in string_and_index()) {
            let mut btree_string = BTreeString::from(&string);
            string.replace_range(..start, "");
            btree_string.truncate_front(start);
            assert_eq!(btree_string.chunks().collect::<String>(), string);
        }

        #[test]
        fn truncate_back((mut string, end) in string_and_index()) {
            let mut btree_string = BTreeString::from(&string);
            string.truncate(end);
            btree_string.truncate_back(end);
            assert_eq!(btree_string.chunks().collect::<String>(), string);
        }

        #[test]
        fn cmp(string_0 in string(), string_1 in string()) {
            let btree_string_0 = BTreeString::from(&string_0);
            let btree_string_1 = BTreeString::from(&string_1);
            assert_eq!(btree_string_0.cmp(&btree_string_1), string_0.cmp(&string_1));
        }

        #[test]
        fn slice_to_btree_string((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.to_btree_string().chunks().collect::<String>(),
                string_slice,
            );
        }

        #[test]
        fn slice_is_empty((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(btree_string_slice.is_empty(), string_slice.is_empty());
        }

        #[test]
        fn slice_len((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(btree_string_slice.len(), string_slice.len());
        }

        #[test]
        fn slice_char_len((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(btree_string_slice.char_len(), string_slice.count_chars());
        }

        #[test]
        fn slice_line_len((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(btree_string_slice.line_len(), string_slice.count_line_breaks() + 1);
        }

        #[test]
        fn slice_is_char_boundary((string, range, index) in string_and_range_and_unaligned_index()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.is_char_boundary(index),
                string_slice.is_char_boundary(index),
            );
        }

        #[test]
        fn slice_index_to_char_index((string, range, index) in string_and_range_and_index()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.index_to_char_index(index),
                string_slice[..index].count_chars(),
            );
        }

        #[test]
        fn slice_index_to_line_index((string, range, index) in string_and_range_and_index()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.index_to_line_index(index),
                string_slice[..index].count_line_breaks(),
            );
        }

        #[test]
        fn slice_char_index_to_index((string, range, char_index) in string_and_range_and_char_index()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.char_index_to_index(char_index),
                string_slice.char_index_to_index(char_index),
            );
        }

        #[test]
        fn slice_line_index_to_index((string, range, line_index) in string_and_range_and_line_index()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.line_index_to_index(line_index),
                string_slice.line_index_to_index(line_index),
            );
        }

        #[test]
        fn slice_chunks((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(btree_string_slice.chunks().collect::<String>(), string_slice);
        }

        #[test]
        fn slice_chunks_rev((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice
                    .chunks_rev()
                    .map(|chunk| chunk.chars().rev().collect::<String>())
                    .collect::<String>(),
                string_slice.chars().rev().collect::<String>(),
            );
        }

        #[test]
        fn slice_bytes((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.bytes().collect::<Vec<_>>(),
                string_slice.bytes().collect::<Vec<_>>(),
            );
        }

        #[test]
        fn slice_bytes_rev((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.bytes_rev().collect::<Vec<_>>(),
                string_slice.bytes().rev().collect::<Vec<_>>()
            );
        }

        #[test]
        fn slice_chars((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.chars().collect::<Vec<_>>(),
                string_slice.chars().collect::<Vec<_>>(),
            );
        }

        #[test]
        fn slice_chars_rev((string, range) in string_and_range()) {
            let string_slice = &string[range.clone()];
            let btree_string = BTreeString::from(&string);
            let btree_string_slice = btree_string.slice(range);
            assert_eq!(
                btree_string_slice.chars_rev().collect::<Vec<_>>(),
                string_slice.chars().rev().collect::<Vec<_>>()
            );
        }

        #[test]
        fn slice_cmp((string_0, range_0) in string_and_range(), (string_1, range_1) in string_and_range()) {
            let string_slice_0 = &string_0[range_0.clone()];
            let btree_string_0 = BTreeString::from(&string_0);
            let btree_string_slice_0 = btree_string_0.slice(range_0);
            let string_slice_1 = &string_1[range_1.clone()];
            let btree_string_1 = BTreeString::from(&string_1);
            let btree_string_slice_1 = btree_string_1.slice(range_1);
            assert_eq!(
                btree_string_slice_0.cmp(&btree_string_slice_1),
                string_slice_0.cmp(&string_slice_1)
            );
        }
    }
}
