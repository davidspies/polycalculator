pub(super) struct Stream<'a>(&'a str);

impl<'a> Stream<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Stream(input)
    }

    fn trim_start(&mut self) {
        self.0 = self.0.trim_start();
    }

    pub(super) fn take_char(&mut self, arg: char) -> bool {
        assert!(!arg.is_ascii_alphanumeric());
        self.trim_start();
        if self.0.starts_with(arg) {
            self.0 = &self.0[arg.len_utf8()..];
            true
        } else {
            false
        }
    }

    pub(super) fn parse_all_matching(&mut self, f: impl Fn(&char) -> bool) -> &'a str {
        self.trim_start();
        let mut end = 0;
        for c in self.0.chars() {
            if f(&c) {
                end += c.len_utf8();
            } else {
                break;
            }
        }
        let output = &self.0[..end];
        self.0 = &self.0[end..];
        output
    }

    pub(super) fn peek_char(&mut self) -> Option<char> {
        self.trim_start();
        self.0.chars().next()
    }

    pub(super) fn finish(mut self) -> Result<(), &'a str> {
        self.trim_start();
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.0)
        }
    }
}
