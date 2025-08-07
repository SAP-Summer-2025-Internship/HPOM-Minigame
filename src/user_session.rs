#[derive(Debug, Clone)]
pub struct UserSession {
    button_presses: Vec<String>,
    current_page: usize,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidButton(String, Vec<String>), // button, allowed_buttons
    InvalidPage(usize),
    NoTransitionDefined(usize),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

impl UserSession {
    /// Consumes self and returns a pseudo-document string describing the user's flow
    pub fn to_doc_string(self) -> String {
        let mut doc = String::new();
        doc.push_str("User Session Flow Summary:\n");
        doc.push_str(&format!("Final page reached: {}\n", self.current_page));
        if self.button_presses.is_empty() {
            doc.push_str("No buttons were pressed.\n");
        } else {
            doc.push_str("Button presses in order:\n");
            for (i, btn) in self.button_presses.iter().enumerate() {
                doc.push_str(&format!("  {}. {}\n", i + 1, btn));
            }
        }
        // Optionally, add a human-readable summary
        doc.push_str("\nSummary:\n");
        match self.button_presses.get(0).map(|s| s.as_str()) {
            Some("start") => doc.push_str("- User started the flow.\n"),
            _ => doc.push_str("- User did not start with the expected button.\n"),
        }
        if let Some(role) = self.button_presses.get(1) {
            doc.push_str(&format!("- Chose role: {}\n", role));
        }
        if self.button_presses.contains(&"mc".to_string()) {
            doc.push_str("- Took the multiple choice path.\n");
        }
        if self.button_presses.contains(&"tf".to_string()) {
            doc.push_str("- Took the true/false path.\n");
        }
        if self.current_page == 9 {
            doc.push_str("- User completed the flow and reached the final page.\n");
        }
        doc
    }
    pub fn new() -> Self {
        Self { 
            button_presses: Vec::new(),
            current_page: 1,
        }
    }
    
    /// Process a button press and return the next page if valid
    /// HARDCODED FLOW LOGIC:
    /// 1. Page 1: "start" -> Page 2
    /// 2. Page 2: "pm"|"ux"|"engi"|"dm" -> Page 3
    /// 3. Page 3: "mc" -> Page 4, "tf" -> Page 5
    /// 4. Page 4: "4a"|"4b"|"4c"|"4d" -> Page 6 (only if "mc" was pressed)
    /// 5. Page 5: "5t"|"5f" -> Page 7 (only if "tf" was pressed)
    /// 6. Page 6: "6a"|"6b"|"6c"|"6d" -> Page 8 (only if "mc" was pressed)
    /// 7. Page 7: "7t"|"7f" -> Page 8 (only if "tf" was pressed)
    /// 8. Page 8: "trophy" -> Page 9
    /// 9. Page 9: No button presses allowed
    pub fn process_button_press(&mut self, button: &str) -> ValidationResult<usize> {
        let next_page = match self.current_page {
            1 => {
                // Page 1: Only "start" allowed
                if button == "start" {
                    Ok(2)
                } else {
                    Err(ValidationError::InvalidButton(
                        button.to_string(), 
                        vec!["start".to_string()]
                    ))
                }
            },
            
            2 => {
                // Page 2: Role selection
                match button {
                    "pm" | "ux" | "engi" | "dm" => Ok(3),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["pm".to_string(), "ux".to_string(), "engi".to_string(), "dm".to_string()]
                    ))
                }
            },
            
            3 => {
                // Page 3: Multiple choice or True/False selection
                match button {
                    "mc" => Ok(4),
                    "tf" => Ok(5),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["mc".to_string(), "tf".to_string()]
                    ))
                }
            },
            
            4 => {
                // Page 4: Multiple choice questions (only if "mc" was pressed)
                if !self.button_presses.contains(&"mc".to_string()) {
                    return Err(ValidationError::InvalidPage(4));
                }
                match button {
                    "4a" | "4b" | "4c" | "4d" => Ok(6),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["4a".to_string(), "4b".to_string(), "4c".to_string(), "4d".to_string()]
                    ))
                }
            },
            
            5 => {
                // Page 5: True/False questions (only if "tf" was pressed)
                if !self.button_presses.contains(&"tf".to_string()) {
                    return Err(ValidationError::InvalidPage(5));
                }
                match button {
                    "5t" | "5f" => Ok(7),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["5t".to_string(), "5f".to_string()]
                    ))
                }
            },
            
            6 => {
                // Page 6: More multiple choice questions (only if "mc" was pressed)
                if !self.button_presses.contains(&"mc".to_string()) {
                    return Err(ValidationError::InvalidPage(6));
                }
                match button {
                    "6a" | "6b" | "6c" | "6d" => Ok(8),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["6a".to_string(), "6b".to_string(), "6c".to_string(), "6d".to_string()]
                    ))
                }
            },
            
            7 => {
                // Page 7: More true/false questions (only if "tf" was pressed)
                if !self.button_presses.contains(&"tf".to_string()) {
                    return Err(ValidationError::InvalidPage(7));
                }
                match button {
                    "7t" | "7f" => Ok(8),
                    _ => Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["7t".to_string(), "7f".to_string()]
                    ))
                }
            },
            
            8 => {
                // Page 8: Trophy button
                if button == "trophy" {
                    Ok(9)
                } else {
                    Err(ValidationError::InvalidButton(
                        button.to_string(),
                        vec!["trophy".to_string()]
                    ))
                }
            },
            
            9 => {
                // Page 9: No button presses allowed
                Err(ValidationError::InvalidButton(
                    button.to_string(),
                    vec![] // No buttons allowed
                ))
            },
            
            _ => Err(ValidationError::NoTransitionDefined(self.current_page))
        }?;
        
        // If we get here, the button press was valid
        self.button_presses.push(button.to_string());
        self.current_page = next_page;
        
        Ok(next_page)
    }
    
    /// Get current page
    pub fn current_page(&self) -> usize {
        self.current_page
    }
    
    /// Get all button presses
    pub fn button_presses(&self) -> &[String] {
        &self.button_presses
    }
}
