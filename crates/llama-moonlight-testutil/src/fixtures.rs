use crate::TestUtilError;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::{tempdir, TempDir};

/// HTML fixture for a simple login page
pub const HTML_LOGIN_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Login Page</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
        .login-form { max-width: 400px; margin: 0 auto; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }
        .form-group { margin-bottom: 15px; }
        label { display: block; margin-bottom: 5px; }
        input[type="text"], input[type="password"] { width: 100%; padding: 8px; box-sizing: border-box; }
        button { padding: 10px 15px; background-color: #4CAF50; color: white; border: none; cursor: pointer; }
        .error { color: red; display: none; }
    </style>
</head>
<body>
    <div class="login-form">
        <h2>Login</h2>
        <form id="login-form">
            <div class="form-group">
                <label for="username">Username:</label>
                <input type="text" id="username" name="username" required>
            </div>
            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required>
            </div>
            <div class="error" id="error-message">Invalid username or password</div>
            <div class="form-group">
                <button type="submit">Login</button>
            </div>
        </form>
    </div>
    <script>
        document.getElementById('login-form').addEventListener('submit', function(e) {
            e.preventDefault();
            
            const username = document.getElementById('username').value;
            const password = document.getElementById('password').value;
            
            // Simple validation
            if (username === 'admin' && password === 'password') {
                window.location.href = 'dashboard.html';
            } else {
                document.getElementById('error-message').style.display = 'block';
            }
        });
    </script>
</body>
</html>
"#;

/// HTML fixture for a simple dashboard page
pub const HTML_DASHBOARD_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
        .dashboard { max-width: 800px; margin: 0 auto; }
        .header { display: flex; justify-content: space-between; align-items: center; }
        .card { border: 1px solid #ddd; border-radius: 5px; padding: 15px; margin-bottom: 15px; }
        .logout { color: blue; cursor: pointer; }
    </style>
</head>
<body>
    <div class="dashboard">
        <div class="header">
            <h2>Welcome, Admin</h2>
            <span class="logout" id="logout">Logout</span>
        </div>
        <div class="card">
            <h3>Statistics</h3>
            <p>Users: 125</p>
            <p>Active Sessions: 42</p>
            <p>New Registrations Today: 5</p>
        </div>
        <div class="card">
            <h3>Recent Activity</h3>
            <ul id="activity-list">
                <li>User John logged in (2 minutes ago)</li>
                <li>User Sarah updated profile (15 minutes ago)</li>
                <li>New user registered: Alex (1 hour ago)</li>
            </ul>
        </div>
    </div>
    <script>
        document.getElementById('logout').addEventListener('click', function() {
            window.location.href = 'login.html';
        });
        
        // Simulate adding a new activity
        setTimeout(function() {
            const list = document.getElementById('activity-list');
            const newItem = document.createElement('li');
            newItem.textContent = 'User David downloaded report (just now)';
            list.insertBefore(newItem, list.firstChild);
        }, 3000);
    </script>
</body>
</html>
"#;

/// A fixture that creates HTML files for testing
pub struct HtmlFixture {
    /// Temp directory for storing the files
    pub temp_dir: TempDir,
    /// Map of created files (name -> path)
    pub files: std::collections::HashMap<String, PathBuf>,
}

impl HtmlFixture {
    /// Create a new HtmlFixture with pre-defined HTML files
    pub fn new() -> Result<Self, TestUtilError> {
        let temp_dir = tempdir()?;
        let mut fixture = Self {
            temp_dir,
            files: std::collections::HashMap::new(),
        };
        
        // Create login page
        fixture.create_file("login.html", HTML_LOGIN_PAGE)?;
        
        // Create dashboard page
        fixture.create_file("dashboard.html", HTML_DASHBOARD_PAGE)?;
        
        Ok(fixture)
    }
    
    /// Create a custom file in the fixture
    pub fn create_file(&mut self, name: &str, content: &str) -> Result<PathBuf, TestUtilError> {
        let file_path = self.temp_dir.path().join(name);
        
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        
        self.files.insert(name.to_string(), file_path.clone());
        
        Ok(file_path)
    }
    
    /// Get the full path to a file
    pub fn get_file_path(&self, name: &str) -> Option<PathBuf> {
        self.files.get(name).cloned()
    }
    
    /// Get the file URL (file:// scheme)
    pub fn get_file_url(&self, name: &str) -> Option<String> {
        self.get_file_path(name).map(|path| {
            format!("file://{}", path.to_string_lossy())
        })
    }
}

/// JSON fixture for a response containing user data
pub const JSON_USERS_RESPONSE: &str = r#"
{
  "users": [
    {
      "id": 1,
      "name": "John Doe",
      "email": "john@example.com",
      "role": "admin",
      "active": true
    },
    {
      "id": 2,
      "name": "Jane Smith",
      "email": "jane@example.com",
      "role": "user",
      "active": true
    },
    {
      "id": 3,
      "name": "Bob Johnson",
      "email": "bob@example.com",
      "role": "user",
      "active": false
    }
  ],
  "total": 3,
  "page": 1,
  "per_page": 10
}
"#;

/// JSON fixture for an authentication request
pub const JSON_AUTH_REQUEST: &str = r#"
{
  "username": "admin",
  "password": "password123"
}
"#;

/// JSON fixture for a successful authentication response
pub const JSON_AUTH_RESPONSE_SUCCESS: &str = r#"
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
  "user": {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "role": "admin"
  }
}
"#;

/// JSON fixture for a failed authentication response
pub const JSON_AUTH_RESPONSE_FAILURE: &str = r#"
{
  "success": false,
  "error": "Invalid username or password"
}
"#;

/// CSS fixture for a simple stylesheet
pub const CSS_STYLESHEET: &str = r#"
body {
    font-family: 'Arial', sans-serif;
    margin: 0;
    padding: 0;
    background-color: #f4f4f4;
}

.container {
    width: 80%;
    margin: 0 auto;
    padding: 20px;
}

header {
    background-color: #35424a;
    color: white;
    padding: 20px 0;
    text-align: center;
}

nav ul {
    list-style: none;
    padding: 0;
    display: flex;
    justify-content: center;
}

nav li {
    margin: 0 10px;
}

nav a {
    color: white;
    text-decoration: none;
}

nav a:hover {
    text-decoration: underline;
}

.main-content {
    padding: 20px;
    background-color: white;
    margin-top: 20px;
    border-radius: 5px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
}

footer {
    text-align: center;
    padding: 20px;
    margin-top: 20px;
    background-color: #35424a;
    color: white;
}

@media (max-width: 768px) {
    .container {
        width: 95%;
    }
    
    nav ul {
        flex-direction: column;
    }
    
    nav li {
        margin: 5px 0;
    }
}
"#;

/// JavaScript fixture for a simple script
pub const JS_SCRIPT: &str = r#"
// Simple utility functions
const utils = {
    // DOM manipulation helpers
    dom: {
        // Get element by ID
        getById: function(id) {
            return document.getElementById(id);
        },
        
        // Create a new element with optional attributes and content
        createElement: function(tag, attributes = {}, content = '') {
            const element = document.createElement(tag);
            
            // Set attributes
            for (const [key, value] of Object.entries(attributes)) {
                element.setAttribute(key, value);
            }
            
            // Set content if provided
            if (content) {
                element.textContent = content;
            }
            
            return element;
        },
        
        // Append multiple children to a parent element
        appendChildren: function(parent, children) {
            children.forEach(child => parent.appendChild(child));
            return parent;
        }
    },
    
    // String utilities
    string: {
        // Truncate a string to a maximum length with ellipsis
        truncate: function(str, maxLength) {
            if (str.length <= maxLength) return str;
            return str.substring(0, maxLength) + '...';
        },
        
        // Capitalize the first letter of each word
        capitalize: function(str) {
            return str.replace(/\b\w/g, match => match.toUpperCase());
        }
    },
    
    // Array utilities
    array: {
        // Filter unique items in an array
        unique: function(arr) {
            return [...new Set(arr)];
        },
        
        // Group array items by a key
        groupBy: function(arr, key) {
            return arr.reduce((result, item) => {
                (result[item[key]] = result[item[key]] || []).push(item);
                return result;
            }, {});
        }
    },
    
    // Date/time utilities
    datetime: {
        // Format date to YYYY-MM-DD
        formatDate: function(date) {
            return date.toISOString().split('T')[0];
        },
        
        // Get relative time string (e.g., "2 hours ago")
        timeAgo: function(date) {
            const seconds = Math.floor((new Date() - date) / 1000);
            
            let interval = Math.floor(seconds / 31536000);
            if (interval > 1) return interval + ' years ago';
            
            interval = Math.floor(seconds / 2592000);
            if (interval > 1) return interval + ' months ago';
            
            interval = Math.floor(seconds / 86400);
            if (interval > 1) return interval + ' days ago';
            
            interval = Math.floor(seconds / 3600);
            if (interval > 1) return interval + ' hours ago';
            
            interval = Math.floor(seconds / 60);
            if (interval > 1) return interval + ' minutes ago';
            
            return Math.floor(seconds) + ' seconds ago';
        }
    }
};

// Sample usage
document.addEventListener('DOMContentLoaded', function() {
    console.log('DOM fully loaded');
    
    // Example of creating and appending elements
    const container = utils.dom.getById('container');
    if (container) {
        const header = utils.dom.createElement('header', {}, 'Welcome to the App');
        const paragraph = utils.dom.createElement('p', { class: 'description' }, 
            'This is a sample application to demonstrate JavaScript utilities.');
        
        utils.dom.appendChildren(container, [header, paragraph]);
    }
});
"#;

/// Create a temporary file with the given content and extension
pub fn create_temp_file(content: &str, extension: &str) -> Result<(TempDir, PathBuf), TestUtilError> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join(format!("temp_file.{}", extension));
    
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    
    Ok((temp_dir, file_path))
} 