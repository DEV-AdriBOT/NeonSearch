// Console API implementation for JavaScript
use std::rc::Rc;
use std::cell::RefCell;

pub struct ConsoleAPI {
    output: Rc<RefCell<Vec<String>>>,
}

impl ConsoleAPI {
    pub fn new(output: Rc<RefCell<Vec<String>>>) -> Self {
        Self { output }
    }
    
    pub fn log(&self, message: &str) {
        self.output.borrow_mut().push(format!("[LOG] {}", message));
    }
    
    pub fn error(&self, message: &str) {
        self.output.borrow_mut().push(format!("[ERROR] {}", message));
    }
    
    pub fn warn(&self, message: &str) {
        self.output.borrow_mut().push(format!("[WARN] {}", message));
    }
    
    pub fn info(&self, message: &str) {
        self.output.borrow_mut().push(format!("[INFO] {}", message));
    }
    
    pub fn get_output(&self) -> Vec<String> {
        self.output.borrow().clone()
    }
    
    pub fn clear_output(&self) {
        self.output.borrow_mut().clear();
    }
}