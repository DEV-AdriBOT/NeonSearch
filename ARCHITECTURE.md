# Architecture Documentation

## NeonSearch Browser Architecture

This document provides a detailed overview of the NeonSearch browser architecture, explaining how each component works together to create a functional web browser built entirely from scratch in Rust.

## System Overview

NeonSearch follows a modular, layered architecture that separates concerns and allows for independent development and testing of different browser components.

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interface Layer                     │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │   Tabs      │ Address Bar │ Navigation  │ Bookmarks   │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    Browser Engine Layer                     │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ HTML Parser │ CSS Parser  │   Layout    │  Renderer   │ │
│  │             │             │   Engine    │             │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                 Platform Services Layer                     │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ Networking  │  Security   │ JavaScript  │   Storage   │ │
│  │             │             │   Engine    │             │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. HTML Parser (`src/engine/html_parser.rs`)

The HTML parser is responsible for converting raw HTML text into a structured DOM tree.

**Key Features:**
- **Tokenization**: Breaks HTML into tokens (tags, text, attributes)
- **Tree Building**: Constructs a proper DOM tree with parent-child relationships
- **Error Recovery**: Handles malformed HTML gracefully
- **HTML5 Compliance**: Supports modern HTML5 syntax and elements

**Architecture:**
```rust
HTMLParser -> Tokenizer -> TreeBuilder -> DOMNode
```

**Process Flow:**
1. Input HTML string is tokenized into start tags, end tags, text nodes
2. Tokens are processed to build a hierarchical DOM tree
3. Self-closing tags and void elements are handled correctly
4. Comments and DOCTYPE declarations are processed or ignored as appropriate

### 2. CSS Parser (`src/engine/css_parser.rs`)

Parses CSS stylesheets and converts them into usable style information.

**Key Components:**
- **Selector Parsing**: Handles element, class, ID, and compound selectors
- **Declaration Parsing**: Processes CSS properties and values
- **Value Parsing**: Converts string values to typed CSS values (colors, lengths, keywords)

**Data Structures:**
```rust
Stylesheet {
  rules: Vec<Rule>
}

Rule {
  selectors: Vec<Selector>,
  declarations: Vec<Declaration>
}
```

### 3. Layout Engine (`src/engine/layout.rs`)

Calculates the position and size of elements based on CSS styles and content.

**Layout Types Supported:**
- **Block Layout**: Standard block-level element positioning
- **Inline Layout**: Text and inline element flow
- **Box Model**: Margin, border, padding calculations

**Layout Process:**
1. **Style Resolution**: Match CSS rules to DOM elements
2. **Box Generation**: Create layout boxes for each element
3. **Positioning**: Calculate exact coordinates and dimensions
4. **Flow**: Handle text flow and line breaking

### 4. Rendering Engine (`src/engine/renderer.rs`)

Converts layout information into visual output.

**Rendering Pipeline:**
1. **Display List Generation**: Create list of drawing commands
2. **Painting**: Execute drawing commands to screen/buffer
3. **Optimization**: Minimize redraws and optimize performance

**Supported Drawing Operations:**
- Solid color rectangles
- Text rendering
- Border drawing
- Image display (planned)

### 5. Networking Layer (`src/networking/`)

Handles all network communication for loading web resources.

**Components:**
- **HTTP Client**: Sends HTTP/HTTPS requests
- **Cookie Management**: Stores and manages cookies securely
- **URL Parsing**: Validates and parses URLs
- **Security**: Validates certificates and enforces security policies

**Request Flow:**
```
URL Input -> URL Validation -> HTTP Request -> Response Processing -> Content Delivery
```

### 6. User Interface (`src/ui/`)

Provides the browser's user interface using the egui framework.

**UI Components:**
- **Main Window**: Application container and window management
- **Tab System**: Multiple page support with tab switching
- **Address Bar**: URL input with autocomplete and security indicators
- **Navigation**: Back, forward, reload, home buttons
- **Bookmarks**: Bookmark management and organization

### 7. Security Framework (`src/security/`)

Implements browser security policies and protections.

**Security Features:**
- **Sandboxing**: Isolates web content from system resources
- **Content Security Policy**: Enforces CSP headers
- **Certificate Validation**: Validates HTTPS certificates
- **Same-Origin Policy**: Prevents cross-origin attacks (planned)

## Data Flow

### Page Loading Process

1. **URL Input**: User enters URL in address bar
2. **URL Validation**: Security and format validation
3. **Network Request**: HTTP client fetches content
4. **HTML Parsing**: Raw HTML converted to DOM tree
5. **CSS Processing**: Stylesheets parsed and applied
6. **Layout Calculation**: Element positions calculated
7. **Rendering**: Visual output generated
8. **Display**: Content shown to user

### User Interaction Flow

```
User Input -> UI Event -> Browser Logic -> Engine Update -> Display Update
```

## Threading Model

NeonSearch uses Rust's async/await model for concurrent operations:

- **Main Thread**: UI rendering and user interaction
- **Network Thread**: HTTP requests and downloads
- **Parser Thread**: HTML/CSS parsing (planned)
- **Layout Thread**: Layout calculations (planned)

## Memory Management

Leverages Rust's ownership system for memory safety:

- **Zero-Copy Parsing**: Minimal string copying during parsing
- **Reference Counting**: Shared data structures use `Rc<RefCell<T>>`
- **RAII**: Automatic resource cleanup
- **No Garbage Collection**: Deterministic memory management

## Performance Optimizations

### Current Optimizations
- **Incremental Parsing**: Process HTML as it arrives
- **Lazy Layout**: Only calculate layout when needed
- **Display List Caching**: Reuse rendering commands when possible

### Planned Optimizations
- **Multi-threaded Parsing**: Parallel HTML/CSS processing
- **GPU Acceleration**: Hardware-accelerated rendering
- **Caching**: Aggressive caching of parsed content
- **Compression**: Efficient internal data structures

## Error Handling

NeonSearch uses Rust's `Result<T, E>` type for error handling:

- **Parse Errors**: Graceful handling of malformed HTML/CSS
- **Network Errors**: Retry logic and user-friendly error messages
- **Security Errors**: Clear security violation reporting
- **System Errors**: Platform-specific error handling

## Extensibility

The modular architecture allows for easy extension:

- **Plugin System**: Future support for browser extensions
- **Custom Parsers**: Support for additional content types
- **Theme System**: Customizable UI themes
- **Protocol Handlers**: Support for custom URL schemes

## Testing Strategy

- **Unit Tests**: Individual component testing
- **Integration Tests**: Component interaction testing
- **Performance Tests**: Benchmark critical paths
- **Compatibility Tests**: Cross-platform validation

## Future Enhancements

1. **JavaScript Engine Integration**: V8 or custom JS engine
2. **Advanced Layout**: Flexbox and CSS Grid support
3. **Developer Tools**: Built-in debugging capabilities
4. **Extensions**: Plugin architecture
5. **Performance**: Multi-core utilization
6. **Accessibility**: Screen reader and keyboard navigation support

This architecture provides a solid foundation for a modern web browser while maintaining the flexibility to add new features and optimizations.