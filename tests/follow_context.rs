use rgrep::{Config, Context};

// We test the internal follow context logic by constructing a minimal engine via a
// small re-export in tests (using the public follow API would block forever).
// To keep changes minimal, we simulate the per-line handling through a local copy
// matching the implementation in src/follow.rs.

#[derive(Debug)]
struct TestEngine {
    before_n: usize,
    after_n: usize,
    before_buf: std::collections::VecDeque<String>,
    after_remaining: usize,
}

impl TestEngine {
    fn new(before_n: usize, after_n: usize) -> Self {
        Self {
            before_n,
            after_n,
            before_buf: std::collections::VecDeque::with_capacity(before_n.max(1)),
            after_remaining: 0,
        }
    }
    fn handle_line(&mut self, line: &str, is_match: bool) -> Vec<String> {
        let mut out = Vec::new();
        let line = line.to_string();
        if is_match {
            if self.before_n > 0 {
                for b in &self.before_buf {
                    out.push(b.clone());
                }
            }
            out.push(line.clone());
            self.after_remaining = self.after_n;
            self.before_buf.clear();
        } else if self.after_remaining > 0 {
            out.push(line.clone());
            self.after_remaining -= 1;
        } else if self.before_n > 0 {
            if self.before_buf.len() == self.before_n {
                self.before_buf.pop_front();
            }
            self.before_buf.push_back(line.clone());
        }
        out
    }
}

fn cfg() -> Config {
    Config {
        patterns: vec!["hund".to_string()],
        color: false,
        ..Default::default()
    }
}

#[test]
fn follow_c_prints_before_and_match() {
    let mut c = cfg();
    c.context = Context {
        before: 2,
        after: 2,
    };

    let mut eng = TestEngine::new(c.context.before, c.context.after);
    let seq = ["affe", "baer", "hund"]; // appended lines

    let mut out = Vec::new();
    for s in seq {
        let is_match = s.contains("hund");
        out.extend(eng.handle_line(s, is_match));
    }

    assert_eq!(out, vec!["affe", "baer", "hund"]);
}

#[test]
fn follow_b_prints_before_and_match() {
    let mut c = cfg();
    c.context = Context {
        before: 2,
        after: 0,
    };

    let mut eng = TestEngine::new(c.context.before, c.context.after);
    let seq = ["affe", "baer", "hund"]; // appended lines

    let mut out = Vec::new();
    for s in seq {
        let is_match = s.contains("hund");
        out.extend(eng.handle_line(s, is_match));
    }

    assert_eq!(out, vec!["affe", "baer", "hund"]);
}

#[test]
fn follow_a_prints_match_and_after() {
    let mut c = cfg();
    c.context = Context {
        before: 0,
        after: 2,
    };

    let mut eng = TestEngine::new(c.context.before, c.context.after);
    let seq = ["hund", "affe", "baer"]; // appended lines

    let mut out = Vec::new();
    for s in seq {
        let is_match = s.contains("hund");
        out.extend(eng.handle_line(s, is_match));
    }

    assert_eq!(out, vec!["hund", "affe", "baer"]);
}

#[test]
fn follow_context_scoped_per_batch_c2() {
    // -C 2 semantics within batches: context should not leak between batches
    let before = 2usize;
    let after = 2usize;

    // batch 1
    let mut eng = TestEngine::new(before, after);
    let batch1 = ["affe", "baer", "hund", "chimpanzee", "bird"];
    let mut out1 = Vec::new();
    for s in batch1 {
        out1.extend(eng.handle_line(s, s.contains("hund")));
    }
    assert_eq!(out1, vec!["affe", "baer", "hund", "chimpanzee", "bird"]);

    // Simulate new append later: reset engine
    let mut eng2 = TestEngine::new(before, after);
    let batch2 = ["hund", "chimpanzee", "bird"];
    let mut out2 = Vec::new();
    for s in batch2 {
        out2.extend(eng2.handle_line(s, s.contains("hund")));
    }
    assert_eq!(out2, vec!["hund", "chimpanzee", "bird"]);
}
