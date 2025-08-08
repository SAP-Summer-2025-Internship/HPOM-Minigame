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
        // Role selection (Page 2)
        let role = self.button_presses.get(1).map(|s| match s.as_str() {
            "pm" => "Product Manager",
            "ux" => "UX Designer",
            "engi" => "Engineer",
            "dm" => "Deveveloper Manager",
            _ => s.as_str(),
        });

        // Question type (Page 3)
        let qtype = self.button_presses.get(2).map(|s| match s.as_str() {
            "mc" => "Multiple Choice",
            "tf" => "True/False",
            _ => s.as_str(),
        });

        // Multiple Choice answers (Pages 4, 6)
        let mut mc_answers = vec![];
        if let Some(qtype) = qtype {
            if qtype == "Multiple Choice" {
                // Page 4
                if let Some(ans) = self.button_presses.get(3) {
                    let team_size = match ans.as_str() {
                        "4a" => "3-5 people",
                        "4b" => "6-8 people",
                        "4c" => "9-12 people",
                        "4d" => "13-15 people",
                        _ => "(unknown)",
                    };
                    mc_answers.push(format!("Preferred team size: {team_size}"));
                }
                // Page 6
                if let Some(ans) = self.button_presses.get(4) {
                    let role_pref = match ans.as_str() {
                        "6a" => "Product Manager",
                        "6b" => "Developer Manager",
                        "6c" => "Engineer",
                        "6d" => "UX Designer",
                        _ => "(unknown)",
                    };
                    mc_answers.push(format!("Wants to see more: {role_pref}"));
                }
            }
        }

        // True/False answers (Pages 5, 7)
        let mut tf_answers = vec![];
        if let Some(qtype) = qtype {
            if qtype == "True/False" {
                // Page 5
                if let Some(ans) = self.button_presses.get(3) {
                    let resp = match ans.as_str() {
                        "5t" => "True",
                        "5f" => "False",
                        _ => "(unknown)",
                    };
                    tf_answers.push(format!("Believes HPOM has been live for two years: {resp}"));
                }
                // Page 7
                if let Some(ans) = self.button_presses.get(4) {
                    let resp = match ans.as_str() {
                        "7t" => "True",
                        "7f" => "False",
                        _ => "(unknown)",
                    };
                    tf_answers.push(format!("Not intimidated by Richard Cai: {resp}"));
                }
            }
        }

        // Compose doc string
        doc.push_str("User Response Summary:\n");
        if let Some(role) = role {
            doc.push_str(&format!("- Role: {}\n", role));
        }
        if let Some(qtype) = qtype {
            doc.push_str(&format!("- Question type: {}\n", qtype));
        }
        for ans in mc_answers {
            doc.push_str(&format!("- {}\n", ans));
        }
        for ans in tf_answers {
            doc.push_str(&format!("- {}\n", ans));
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
