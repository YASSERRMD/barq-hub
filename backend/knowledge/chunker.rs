//! Document chunking

use crate::knowledge::Chunk;

pub struct DocumentChunker {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentChunker {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self { chunk_size, chunk_overlap }
    }

    pub fn chunk(&self, document_id: &str, content: &str) -> Vec<Chunk> {
        if content.len() <= self.chunk_size {
            return vec![Chunk::new(document_id, content, 0, content.len(), 0)];
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < content.len() {
            let end = (start + self.chunk_size).min(content.len());
            
            // Try to break at sentence/word boundary
            let actual_end = if end < content.len() {
                self.find_break_point(content, end)
            } else {
                end
            };

            let chunk_content = &content[start..actual_end];
            chunks.push(Chunk::new(document_id, chunk_content, start, actual_end, index));
            
            // Ensure we always advance (avoid infinite loop)
            let next_start = actual_end.saturating_sub(self.chunk_overlap);
            start = if next_start <= start { actual_end } else { next_start };
            
            index += 1;

            if start >= content.len() || chunks.len() > 10000 { break; }
        }

        chunks
    }

    fn find_break_point(&self, content: &str, target: usize) -> usize {
        // Look backwards for sentence end or newline
        let search_start = target.saturating_sub(100);
        let search_range = &content[search_start..target];
        
        for (i, c) in search_range.char_indices().rev() {
            if c == '.' || c == '\n' || c == '!' || c == '?' {
                return search_start + i + 1;
            }
        }
        
        // Fall back to word boundary
        for (i, c) in search_range.char_indices().rev() {
            if c.is_whitespace() {
                return search_start + i + 1;
            }
        }
        
        target
    }
}

impl Default for DocumentChunker {
    fn default() -> Self { Self::new(1000, 200) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_document() {
        let chunker = DocumentChunker::new(100, 20);
        let chunks = chunker.chunk("doc1", "Hello world");
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_large_document() {
        let chunker = DocumentChunker::new(50, 10);
        let content = "A".repeat(200);
        let chunks = chunker.chunk("doc1", &content);
        assert!(chunks.len() > 1);
    }
}
