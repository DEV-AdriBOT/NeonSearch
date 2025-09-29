// Test file for JavaScript engine functionality
use crate::js::JSEngine;
use crate::engine::html_parser;

pub fn test_js_engine() {
    println!("🧪 Testing JavaScript Engine...");
    
    match JSEngine::new() {
        Ok(mut engine) => {
            println!("✅ JavaScript engine created successfully");
            
            // Test 1: console.log with string literal
            match engine.execute(r#"console.log("Hello from NeonSearch JavaScript!")"#) {
                Ok(result) => println!("✅ Console.log test passed: {}", result),
                Err(e) => println!("❌ Console.log test failed: {}", e),
            }
            
            // Test 2: Variable declaration
            match engine.execute(r#"var greeting = "Hello World""#) {
                Ok(result) => println!("✅ Variable declaration test passed: {}", result),
                Err(e) => println!("❌ Variable declaration test failed: {}", e),
            }
            
            // Test 3: Variable access
            match engine.execute("greeting") {
                Ok(result) => println!("✅ Variable access test passed: {}", result),
                Err(e) => println!("❌ Variable access test failed: {}", e),
            }
            
            // Test 4: console.log with variable
            match engine.execute("console.log(greeting)") {
                Ok(result) => println!("✅ Console.log with variable test passed: {}", result),
                Err(e) => println!("❌ Console.log with variable test failed: {}", e),
            }
            
            // Test 5: Number operations
            match engine.execute("var num = 42") {
                Ok(_) => {},
                Err(e) => println!("❌ Number declaration failed: {}", e),
            }
            
            match engine.execute("num") {
                Ok(result) => println!("✅ Number variable test passed: {}", result),
                Err(e) => println!("❌ Number variable test failed: {}", e),
            }
            
            // Test 6: Boolean values
            match engine.execute("var isActive = true") {
                Ok(_) => {},
                Err(e) => println!("❌ Boolean declaration failed: {}", e),
            }
            
            match engine.execute("isActive") {
                Ok(result) => println!("✅ Boolean variable test passed: {}", result),
                Err(e) => println!("❌ Boolean variable test failed: {}", e),
            }
            
            // Show console output
            let console_output = engine.get_console_output();
            if !console_output.is_empty() {
                println!("📋 Console Output:");
                for line in console_output {
                    println!("  {}", line);
                }
            }
            
            println!("🎉 JavaScript engine tests completed!");
        },
        Err(e) => {
            println!("❌ Failed to create JavaScript engine: {}", e);
        }
    }
}

pub fn test_html_with_js() {
    println!("\n🌐 Testing HTML with JavaScript...");
    
    let test_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>JS Test</title>
</head>
<body>
    <h1>JavaScript Test</h1>
    <script>
        console.log("Script tag executed!");
        var msg = "Hello from HTML script";
        console.log(msg);
    </script>
    <p>Content after script</p>
    <script>
        console.log("Second script executed");
        var count = 123;
        console.log("Count: " + count);
    </script>
</body>
</html>
"#;

    match JSEngine::new() {
        Ok(mut engine) => {
            println!("✅ Created JavaScript engine for HTML test");
            
            // Parse HTML with JavaScript execution
            let dom = html_parser::parse_with_js(test_html, &mut Some(engine));
            
            println!("✅ HTML parsed with JavaScript execution");
            println!("📊 DOM structure created with {} top-level nodes", 
                match &dom {
                    crate::engine::dom::DOMNode::Element { children, .. } => children.len(),
                    _ => 0
                });
            
            // Note: Console output should have been captured during parsing
            println!("🎉 HTML with JavaScript test completed!");
        },
        Err(e) => {
            println!("❌ Failed to create JavaScript engine for HTML test: {}", e);
        }
    }
}