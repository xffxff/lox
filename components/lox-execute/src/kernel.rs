pub trait Kernel {
    // Implementation for `print` intrinsic, that prints a line of text.
    fn print(&mut self, text: &str);
}

pub struct BufferKernel {
    buffer: String,
}

impl Default for BufferKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferKernel {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn take_buffer(self) -> String {
        self.buffer
    }
}

impl Kernel for BufferKernel {
    fn print(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.buffer.push('\n');
    }
}

pub struct StdoutKernel;

impl Kernel for StdoutKernel {
    fn print(&mut self, text: &str) {
        print!("{}", text);
    }
}
